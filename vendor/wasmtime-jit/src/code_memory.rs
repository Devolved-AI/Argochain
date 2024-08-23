//! Memory management for executable code.

use crate::subslice_range;
use crate::unwind::UnwindRegistration;
use anyhow::{anyhow, bail, Context, Result};
use object::read::{File, Object, ObjectSection};
use object::ObjectSymbol;
use std::mem;
use std::mem::ManuallyDrop;
use std::ops::Range;
use wasmtime_environ::obj;
use wasmtime_environ::FunctionLoc;
use wasmtime_jit_icache_coherence as icache_coherence;
use wasmtime_runtime::libcalls;
use wasmtime_runtime::{MmapVec, VMTrampoline};

/// Management of executable memory within a `MmapVec`
///
/// This type consumes ownership of a region of memory and will manage the
/// executable permissions of the contained JIT code as necessary.
pub struct CodeMemory {
    // NB: these are `ManuallyDrop` because `unwind_registration` must be
    // dropped first since it refers to memory owned by `mmap`.
    mmap: ManuallyDrop<MmapVec>,
    unwind_registration: ManuallyDrop<Option<UnwindRegistration>>,
    published: bool,
    enable_branch_protection: bool,

    relocations: Vec<(usize, obj::LibCall)>,

    // Ranges within `self.mmap` of where the particular sections lie.
    text: Range<usize>,
    unwind: Range<usize>,
    trap_data: Range<usize>,
    wasm_data: Range<usize>,
    address_map_data: Range<usize>,
    func_name_data: Range<usize>,
    info_data: Range<usize>,
    dwarf: Range<usize>,
}

impl Drop for CodeMemory {
    fn drop(&mut self) {
        // Drop `unwind_registration` before `self.mmap`
        unsafe {
            ManuallyDrop::drop(&mut self.unwind_registration);
            ManuallyDrop::drop(&mut self.mmap);
        }
    }
}

fn _assert() {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<CodeMemory>();
}

impl CodeMemory {
    /// Creates a new `CodeMemory` by taking ownership of the provided
    /// `MmapVec`.
    ///
    /// The returned `CodeMemory` manages the internal `MmapVec` and the
    /// `publish` method is used to actually make the memory executable.
    pub fn new(mmap: MmapVec) -> Result<Self> {
        let obj = File::parse(&mmap[..])
            .with_context(|| "failed to parse internal compilation artifact")?;

        let mut relocations = Vec::new();
        let mut text = 0..0;
        let mut unwind = 0..0;
        let mut enable_branch_protection = None;
        let mut trap_data = 0..0;
        let mut wasm_data = 0..0;
        let mut address_map_data = 0..0;
        let mut func_name_data = 0..0;
        let mut info_data = 0..0;
        let mut dwarf = 0..0;
        for section in obj.sections() {
            let data = section.data()?;
            let name = section.name()?;
            let range = subslice_range(data, &mmap);

            // Double-check that sections are all aligned properly.
            if section.align() != 0 && data.len() != 0 {
                if (data.as_ptr() as u64 - mmap.as_ptr() as u64) % section.align() != 0 {
                    bail!(
                        "section `{}` isn't aligned to {:#x}",
                        section.name().unwrap_or("ERROR"),
                        section.align()
                    );
                }
            }

            match name {
                obj::ELF_WASM_BTI => match data.len() {
                    1 => enable_branch_protection = Some(data[0] != 0),
                    _ => bail!("invalid `{name}` section"),
                },
                ".text" => {
                    text = range;

                    // The text section might have relocations for things like
                    // libcalls which need to be applied, so handle those here.
                    //
                    // Note that only a small subset of possible relocations are
                    // handled. Only those required by the compiler side of
                    // things are processed.
                    for (offset, reloc) in section.relocations() {
                        assert_eq!(reloc.kind(), object::RelocationKind::Absolute);
                        assert_eq!(reloc.encoding(), object::RelocationEncoding::Generic);
                        assert_eq!(usize::from(reloc.size()), std::mem::size_of::<usize>());
                        assert_eq!(reloc.addend(), 0);
                        let sym = match reloc.target() {
                            object::RelocationTarget::Symbol(id) => id,
                            other => panic!("unknown relocation target {other:?}"),
                        };
                        let sym = obj.symbol_by_index(sym).unwrap().name().unwrap();
                        let libcall = obj::LibCall::from_str(sym)
                            .unwrap_or_else(|| panic!("unknown symbol relocation: {sym}"));

                        let offset = usize::try_from(offset).unwrap();
                        relocations.push((offset, libcall));
                    }
                }
                UnwindRegistration::SECTION_NAME => unwind = range,
                obj::ELF_WASM_DATA => wasm_data = range,
                obj::ELF_WASMTIME_ADDRMAP => address_map_data = range,
                obj::ELF_WASMTIME_TRAPS => trap_data = range,
                obj::ELF_NAME_DATA => func_name_data = range,
                obj::ELF_WASMTIME_INFO => info_data = range,
                obj::ELF_WASMTIME_DWARF => dwarf = range,

                _ => log::debug!("ignoring section {name}"),
            }
        }
        Ok(Self {
            mmap: ManuallyDrop::new(mmap),
            unwind_registration: ManuallyDrop::new(None),
            published: false,
            enable_branch_protection: enable_branch_protection
                .ok_or_else(|| anyhow!("missing `{}` section", obj::ELF_WASM_BTI))?,
            text,
            unwind,
            trap_data,
            address_map_data,
            func_name_data,
            dwarf,
            info_data,
            wasm_data,
            relocations,
        })
    }

    /// Returns a reference to the underlying `MmapVec` this memory owns.
    pub fn mmap(&self) -> &MmapVec {
        &self.mmap
    }

    /// Returns the contents of the text section of the ELF executable this
    /// represents.
    pub fn text(&self) -> &[u8] {
        &self.mmap[self.text.clone()]
    }

    /// Returns the contents of the `ELF_WASMTIME_DWARF` section.
    pub fn dwarf(&self) -> &[u8] {
        &self.mmap[self.dwarf.clone()]
    }

    /// Returns the data in the `ELF_NAME_DATA` section.
    pub fn func_name_data(&self) -> &[u8] {
        &self.mmap[self.func_name_data.clone()]
    }

    /// Returns the concatenated list of all data associated with this wasm
    /// module.
    ///
    /// This is used for initialization of memories and all data ranges stored
    /// in a `Module` are relative to the slice returned here.
    pub fn wasm_data(&self) -> &[u8] {
        &self.mmap[self.wasm_data.clone()]
    }

    /// Returns the encoded address map section used to pass to
    /// `wasmtime_environ::lookup_file_pos`.
    pub fn address_map_data(&self) -> &[u8] {
        &self.mmap[self.address_map_data.clone()]
    }

    /// Returns the contents of the `ELF_WASMTIME_INFO` section, or an empty
    /// slice if it wasn't found.
    pub fn wasmtime_info(&self) -> &[u8] {
        &self.mmap[self.info_data.clone()]
    }

    /// Returns the contents of the `ELF_WASMTIME_TRAPS` section, or an empty
    /// slice if it wasn't found.
    pub fn trap_data(&self) -> &[u8] {
        &self.mmap[self.trap_data.clone()]
    }

    /// Returns a `VMTrampoline` function pointer for the given function in the
    /// text section.
    ///
    /// # Unsafety
    ///
    /// This function is unsafe as there's no guarantee that the returned
    /// function pointer is valid.
    pub unsafe fn vmtrampoline(&self, loc: FunctionLoc) -> VMTrampoline {
        let ptr = self.text()[loc.start as usize..][..loc.length as usize].as_ptr();
        mem::transmute::<*const u8, VMTrampoline>(ptr)
    }

    /// Publishes the internal ELF image to be ready for execution.
    ///
    /// This method can only be called once and will panic if called twice. This
    /// will parse the ELF image from the original `MmapVec` and do everything
    /// necessary to get it ready for execution, including:
    ///
    /// * Change page protections from read/write to read/execute.
    /// * Register unwinding information with the OS
    ///
    /// After this function executes all JIT code should be ready to execute.
    pub fn publish(&mut self) -> Result<()> {
        assert!(!self.published);
        self.published = true;

        if self.text().is_empty() {
            return Ok(());
        }

        // The unsafety here comes from a few things:
        //
        // * We're actually updating some page protections to executable memory.
        //
        // * We're registering unwinding information which relies on the
        //   correctness of the information in the first place. This applies to
        //   both the actual unwinding tables as well as the validity of the
        //   pointers we pass in itself.
        unsafe {
            // First, if necessary, apply relocations. This can happen for
            // things like libcalls which happen late in the lowering process
            // that don't go through the Wasm-based libcalls layer that's
            // indirected through the `VMContext`. Note that most modules won't
            // have relocations, so this typically doesn't do anything.
            self.apply_relocations()?;

            // Next freeze the contents of this image by making all of the
            // memory readonly. Nothing after this point should ever be modified
            // so commit everything. For a compiled-in-memory image this will
            // mean IPIs to evict writable mappings from other cores. For
            // loaded-from-disk images this shouldn't result in IPIs so long as
            // there weren't any relocations because nothing should have
            // otherwise written to the image at any point either.
            self.mmap.make_readonly(0..self.mmap.len())?;

            let text = self.text();

            // Clear the newly allocated code from cache if the processor requires it
            //
            // Do this before marking the memory as R+X, technically we should be able to do it after
            // but there are some CPU's that have had errata about doing this with read only memory.
            icache_coherence::clear_cache(text.as_ptr().cast(), text.len())
                .expect("Failed cache clear");

            // Switch the executable portion from readonly to read/execute.
            self.mmap
                .make_executable(self.text.clone(), self.enable_branch_protection)
                .expect("unable to make memory executable");

            // Flush any in-flight instructions from the pipeline
            icache_coherence::pipeline_flush_mt().expect("Failed pipeline flush");

            // With all our memory set up use the platform-specific
            // `UnwindRegistration` implementation to inform the general
            // runtime that there's unwinding information available for all
            // our just-published JIT functions.
            self.register_unwind_info()?;
        }

        Ok(())
    }

    unsafe fn apply_relocations(&mut self) -> Result<()> {
        if self.relocations.is_empty() {
            return Ok(());
        }

        for (offset, libcall) in self.relocations.iter() {
            let offset = self.text.start + offset;
            let libcall = match libcall {
                obj::LibCall::FloorF32 => libcalls::relocs::floorf32 as usize,
                obj::LibCall::FloorF64 => libcalls::relocs::floorf64 as usize,
                obj::LibCall::NearestF32 => libcalls::relocs::nearestf32 as usize,
                obj::LibCall::NearestF64 => libcalls::relocs::nearestf64 as usize,
                obj::LibCall::CeilF32 => libcalls::relocs::ceilf32 as usize,
                obj::LibCall::CeilF64 => libcalls::relocs::ceilf64 as usize,
                obj::LibCall::TruncF32 => libcalls::relocs::truncf32 as usize,
                obj::LibCall::TruncF64 => libcalls::relocs::truncf64 as usize,
                obj::LibCall::FmaF32 => libcalls::relocs::fmaf32 as usize,
                obj::LibCall::FmaF64 => libcalls::relocs::fmaf64 as usize,
            };
            *self.mmap.as_mut_ptr().add(offset).cast::<usize>() = libcall;
        }
        Ok(())
    }

    unsafe fn register_unwind_info(&mut self) -> Result<()> {
        if self.unwind.len() == 0 {
            return Ok(());
        }
        let text = self.text();
        let unwind_info = &self.mmap[self.unwind.clone()];
        let registration =
            UnwindRegistration::new(text.as_ptr(), unwind_info.as_ptr(), unwind_info.len())
                .context("failed to create unwind info registration")?;
        *self.unwind_registration = Some(registration);
        Ok(())
    }
}
