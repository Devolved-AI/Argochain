//! ARM 64-bit Instruction Set Architecture.

use crate::dominator_tree::DominatorTree;
use crate::ir::condcodes::IntCC;
use crate::ir::{Function, Type};
use crate::isa::aarch64::settings as aarch64_settings;
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv;
use crate::isa::{Builder as IsaBuilder, TargetIsa};
use crate::machinst::{
    compile, CompiledCode, CompiledCodeStencil, MachTextSectionBuilder, Reg, SigSet,
    TextSectionBuilder, VCode,
};
use crate::result::CodegenResult;
use crate::settings as shared_settings;
use alloc::{boxed::Box, vec::Vec};
use core::fmt;
use regalloc2::MachineEnv;
use target_lexicon::{Aarch64Architecture, Architecture, OperatingSystem, Triple};

// New backend:
mod abi;
pub mod inst;
mod lower;
pub mod settings;

use inst::create_reg_env;

use self::inst::EmitInfo;

/// An AArch64 backend.
pub struct AArch64Backend {
    triple: Triple,
    flags: shared_settings::Flags,
    isa_flags: aarch64_settings::Flags,
    machine_env: MachineEnv,
}

impl AArch64Backend {
    /// Create a new AArch64 backend with the given (shared) flags.
    pub fn new_with_flags(
        triple: Triple,
        flags: shared_settings::Flags,
        isa_flags: aarch64_settings::Flags,
    ) -> AArch64Backend {
        let machine_env = create_reg_env(&flags);
        AArch64Backend {
            triple,
            flags,
            isa_flags,
            machine_env,
        }
    }

    /// This performs lowering to VCode, register-allocates the code, computes block layout and
    /// finalizes branches. The result is ready for binary emission.
    fn compile_vcode(
        &self,
        func: &Function,
        domtree: &DominatorTree,
    ) -> CodegenResult<(VCode<inst::Inst>, regalloc2::Output)> {
        let emit_info = EmitInfo::new(self.flags.clone());
        let sigs = SigSet::new::<abi::AArch64MachineDeps>(func, &self.flags)?;
        let abi = abi::AArch64Callee::new(func, self, &self.isa_flags, &sigs)?;
        compile::compile::<AArch64Backend>(func, domtree, self, abi, emit_info, sigs)
    }
}

impl TargetIsa for AArch64Backend {
    fn compile_function(
        &self,
        func: &Function,
        domtree: &DominatorTree,
        want_disasm: bool,
    ) -> CodegenResult<CompiledCodeStencil> {
        let (vcode, regalloc_result) = self.compile_vcode(func, domtree)?;

        let emit_result = vcode.emit(
            &regalloc_result,
            want_disasm,
            self.flags.machine_code_cfg_info(),
        );
        let frame_size = emit_result.frame_size;
        let value_labels_ranges = emit_result.value_labels_ranges;
        let buffer = emit_result.buffer.finish();
        let sized_stackslot_offsets = emit_result.sized_stackslot_offsets;
        let dynamic_stackslot_offsets = emit_result.dynamic_stackslot_offsets;

        if let Some(disasm) = emit_result.disasm.as_ref() {
            log::debug!("disassembly:\n{}", disasm);
        }

        Ok(CompiledCodeStencil {
            buffer,
            frame_size,
            vcode: emit_result.disasm,
            value_labels_ranges,
            sized_stackslot_offsets,
            dynamic_stackslot_offsets,
            bb_starts: emit_result.bb_offsets,
            bb_edges: emit_result.bb_edges,
            alignment: emit_result.alignment,
        })
    }

    fn name(&self) -> &'static str {
        "aarch64"
    }

    fn triple(&self) -> &Triple {
        &self.triple
    }

    fn flags(&self) -> &shared_settings::Flags {
        &self.flags
    }

    fn machine_env(&self) -> &MachineEnv {
        &self.machine_env
    }

    fn isa_flags(&self) -> Vec<shared_settings::Value> {
        self.isa_flags.iter().collect()
    }

    fn is_branch_protection_enabled(&self) -> bool {
        self.isa_flags.use_bti()
    }

    fn dynamic_vector_bytes(&self, _dyn_ty: Type) -> u32 {
        16
    }

    fn unsigned_add_overflow_condition(&self) -> IntCC {
        // Unsigned `>=`; this corresponds to the carry flag set on aarch64, which happens on
        // overflow of an add.
        IntCC::UnsignedGreaterThanOrEqual
    }

    #[cfg(feature = "unwind")]
    fn emit_unwind_info(
        &self,
        result: &CompiledCode,
        kind: crate::machinst::UnwindInfoKind,
    ) -> CodegenResult<Option<crate::isa::unwind::UnwindInfo>> {
        use crate::isa::unwind::UnwindInfo;
        use crate::machinst::UnwindInfoKind;
        Ok(match kind {
            UnwindInfoKind::SystemV => {
                let mapper = self::inst::unwind::systemv::RegisterMapper;
                Some(UnwindInfo::SystemV(
                    crate::isa::unwind::systemv::create_unwind_info_from_insts(
                        &result.buffer.unwind_info[..],
                        result.buffer.data().len(),
                        &mapper,
                    )?,
                ))
            }
            UnwindInfoKind::Windows => {
                // TODO: support Windows unwind info on AArch64
                None
            }
            _ => None,
        })
    }

    #[cfg(feature = "unwind")]
    fn create_systemv_cie(&self) -> Option<gimli::write::CommonInformationEntry> {
        let is_apple_os = match self.triple.operating_system {
            OperatingSystem::Darwin
            | OperatingSystem::Ios
            | OperatingSystem::MacOSX { .. }
            | OperatingSystem::Tvos => true,
            _ => false,
        };

        if self.isa_flags.sign_return_address()
            && self.isa_flags.sign_return_address_with_bkey()
            && !is_apple_os
        {
            unimplemented!("Specifying that the B key is used with pointer authentication instructions in the CIE is not implemented.");
        }

        Some(inst::unwind::systemv::create_cie())
    }

    fn text_section_builder(&self, num_funcs: usize) -> Box<dyn TextSectionBuilder> {
        Box::new(MachTextSectionBuilder::<inst::Inst>::new(num_funcs))
    }

    #[cfg(feature = "unwind")]
    fn map_regalloc_reg_to_dwarf(&self, reg: Reg) -> Result<u16, systemv::RegisterMappingError> {
        inst::unwind::systemv::map_reg(reg).map(|reg| reg.0)
    }

    fn function_alignment(&self) -> u32 {
        // We use 32-byte alignment for performance reasons, but for correctness we would only need
        // 4-byte alignment.
        32
    }

    #[cfg(feature = "disas")]
    fn to_capstone(&self) -> Result<capstone::Capstone, capstone::Error> {
        use capstone::prelude::*;
        let mut cs = Capstone::new()
            .arm64()
            .mode(arch::arm64::ArchMode::Arm)
            .build()?;
        // AArch64 uses inline constants rather than a separate constant pool right now.
        // Without this option, Capstone will stop disassembling as soon as it sees
        // an inline constant that is not also a valid instruction. With this option,
        // Capstone will print a `.byte` directive with the bytes of the inline constant
        // and continue to the next instruction.
        cs.set_skipdata(true)?;
        Ok(cs)
    }

    fn has_native_fma(&self) -> bool {
        true
    }
}

impl fmt::Display for AArch64Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MachBackend")
            .field("name", &self.name())
            .field("triple", &self.triple())
            .field("flags", &format!("{}", self.flags()))
            .finish()
    }
}

/// Create a new `isa::Builder`.
pub fn isa_builder(triple: Triple) -> IsaBuilder {
    assert!(triple.architecture == Architecture::Aarch64(Aarch64Architecture::Aarch64));
    IsaBuilder {
        triple,
        setup: aarch64_settings::builder(),
        constructor: |triple, shared_flags, builder| {
            let isa_flags = aarch64_settings::Flags::new(&shared_flags, builder);
            let backend = AArch64Backend::new_with_flags(triple, shared_flags, isa_flags);
            Ok(backend.wrapped())
        },
    }
}
