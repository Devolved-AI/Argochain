//! Implements a registry of modules for a store.

use crate::code::CodeObject;
#[cfg(feature = "component-model")]
use crate::component::Component;
use crate::{FrameInfo, Module, Trap};
use once_cell::sync::Lazy;
use std::collections::btree_map::Entry;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use wasmtime_jit::CodeMemory;
use wasmtime_runtime::{ModuleInfo, VMCallerCheckedFuncRef, VMTrampoline};

/// Used for registering modules with a store.
///
/// Note that the primary reason for this registry is to ensure that everything
/// in `Module` is kept alive for the duration of a `Store`. At this time we
/// need "basically everything" within a `Module` to stay alive once it's
/// instantiated within a store. While there's some smaller portions that could
/// theoretically be omitted as they're not needed by the store they're
/// currently small enough to not worry much about.
#[derive(Default)]
pub struct ModuleRegistry {
    // Keyed by the end address of a `CodeObject`.
    //
    // The value here is the start address and the information about what's
    // loaded at that address.
    loaded_code: BTreeMap<usize, (usize, LoadedCode)>,

    // Preserved for keeping data segments alive or similar
    modules_without_code: Vec<Module>,
}

struct LoadedCode {
    /// Representation of loaded code which could be either a component or a
    /// module.
    code: Arc<CodeObject>,

    /// Modules found within `self.code`, keyed by start address here of the
    /// address of the first function in the module.
    modules: BTreeMap<usize, Module>,
}

impl ModuleRegistry {
    /// Fetches information about a registered module given a program counter value.
    pub fn lookup_module(&self, pc: usize) -> Option<&dyn ModuleInfo> {
        self.module(pc).map(|(m, _)| m.module_info())
    }

    fn code(&self, pc: usize) -> Option<(&LoadedCode, usize)> {
        let (end, (start, code)) = self.loaded_code.range(pc..).next()?;
        if pc < *start || *end < pc {
            return None;
        }
        Some((code, pc - *start))
    }

    fn module(&self, pc: usize) -> Option<(&Module, usize)> {
        let (code, offset) = self.code(pc)?;
        Some((code.module(pc)?, offset))
    }

    /// Registers a new module with the registry.
    pub fn register_module(&mut self, module: &Module) {
        self.register(module.code_object(), Some(module))
    }

    #[cfg(feature = "component-model")]
    pub fn register_component(&mut self, component: &Component) {
        self.register(component.code_object(), None)
    }

    /// Registers a new module with the registry.
    fn register(&mut self, code: &Arc<CodeObject>, module: Option<&Module>) {
        let text = code.code_memory().text();

        // If there's not actually any functions in this module then we may
        // still need to preserve it for its data segments. Instances of this
        // module will hold a pointer to the data stored in the module itself,
        // and for schemes that perform lazy initialization which could use the
        // module in the future. For that reason we continue to register empty
        // modules and retain them.
        if text.is_empty() {
            self.modules_without_code.extend(module.cloned());
            return;
        }

        // The module code range is exclusive for end, so make it inclusive as
        // it may be a valid PC value
        let start_addr = text.as_ptr() as usize;
        let end_addr = start_addr + text.len() - 1;

        // If this module is already present in the registry then that means
        // it's either an overlapping image, for example for two modules
        // found within a component, or it's a second instantiation of the same
        // module. Delegate to `push_module` to find out.
        if let Some((other_start, prev)) = self.loaded_code.get_mut(&end_addr) {
            assert_eq!(*other_start, start_addr);
            if let Some(module) = module {
                prev.push_module(module);
            }
            return;
        }

        // Assert that this module's code doesn't collide with any other
        // registered modules
        if let Some((_, (prev_start, _))) = self.loaded_code.range(start_addr..).next() {
            assert!(*prev_start > end_addr);
        }
        if let Some((prev_end, _)) = self.loaded_code.range(..=start_addr).next_back() {
            assert!(*prev_end < start_addr);
        }

        let mut item = LoadedCode {
            code: code.clone(),
            modules: Default::default(),
        };
        if let Some(module) = module {
            item.push_module(module);
        }
        let prev = self.loaded_code.insert(end_addr, (start_addr, item));
        assert!(prev.is_none());
    }

    /// Looks up a trampoline from an anyfunc.
    pub fn lookup_trampoline(&self, anyfunc: &VMCallerCheckedFuncRef) -> Option<VMTrampoline> {
        let (code, _offset) = self.code(anyfunc.func_ptr.as_ptr() as usize)?;
        code.code.signatures().trampoline(anyfunc.type_index)
    }

    /// Fetches trap information about a program counter in a backtrace.
    pub fn lookup_trap_code(&self, pc: usize) -> Option<Trap> {
        let (code, offset) = self.code(pc)?;
        wasmtime_environ::lookup_trap_code(code.code.code_memory().trap_data(), offset)
    }

    /// Fetches frame information about a program counter in a backtrace.
    ///
    /// Returns an object if this `pc` is known to some previously registered
    /// module, or returns `None` if no information can be found. The first
    /// boolean returned indicates whether the original module has unparsed
    /// debug information due to the compiler's configuration. The second
    /// boolean indicates whether the engine used to compile this module is
    /// using environment variables to control debuginfo parsing.
    pub(crate) fn lookup_frame_info(&self, pc: usize) -> Option<(FrameInfo, &Module)> {
        let (module, offset) = self.module(pc)?;
        let info = FrameInfo::new(module, offset)?;
        Some((info, module))
    }
}

impl LoadedCode {
    fn push_module(&mut self, module: &Module) {
        let func = match module.compiled_module().finished_functions().next() {
            Some((_, func)) => func,
            // There are no compiled functions in this module so there's no
            // need to push onto `self.modules` which is only used for frame
            // information lookup for a trap which only symbolicates defined
            // functions.
            None => return,
        };
        let start = func.as_ptr() as usize;

        match self.modules.entry(start) {
            // This module is already present, and it should be the same as
            // `module`.
            Entry::Occupied(m) => {
                debug_assert!(Arc::ptr_eq(&module.inner, &m.get().inner));
            }
            // This module was not already present, so now it's time to insert.
            Entry::Vacant(v) => {
                v.insert(module.clone());
            }
        }
    }

    fn module(&self, pc: usize) -> Option<&Module> {
        // The `modules` map is keyed on the start address of the first
        // function in the module, so find the first module whose start address
        // is less than the `pc`. That may be the wrong module but lookup
        // within the module should fail in that case.
        let (_start, module) = self.modules.range(..=pc).next_back()?;
        Some(module)
    }
}

// This is the global code registry that stores information for all loaded code
// objects that are currently in use by any `Store` in the current process.
//
// The purpose of this map is to be called from signal handlers to determine
// whether a program counter is a wasm trap or not. Specifically macOS has
// no contextual information about the thread available, hence the necessity
// for global state rather than using thread local state.
//
// This is similar to `ModuleRegistry` except that it has less information and
// supports removal. Any time anything is registered with a `ModuleRegistry`
// it is also automatically registered with the singleton global module
// registry. When a `ModuleRegistry` is destroyed then all of its entries
// are removed from the global registry.
static GLOBAL_CODE: Lazy<RwLock<GlobalRegistry>> = Lazy::new(Default::default);

type GlobalRegistry = BTreeMap<usize, (usize, Arc<CodeMemory>)>;

/// Returns whether the `pc`, according to globally registered information,
/// is a wasm trap or not.
pub fn is_wasm_trap_pc(pc: usize) -> bool {
    let (code, text_offset) = {
        let all_modules = GLOBAL_CODE.read().unwrap();

        let (end, (start, module)) = match all_modules.range(pc..).next() {
            Some(info) => info,
            None => return false,
        };
        if pc < *start || *end < pc {
            return false;
        }
        (module.clone(), pc - *start)
    };

    wasmtime_environ::lookup_trap_code(code.trap_data(), text_offset).is_some()
}

/// Registers a new region of code.
///
/// Must not have been previously registered and must be `unregister`'d to
/// prevent leaking memory.
///
/// This is required to enable traps to work correctly since the signal handler
/// will lookup in the `GLOBAL_CODE` list to determine which a particular pc
/// is a trap or not.
pub fn register_code(code: &Arc<CodeMemory>) {
    let text = code.text();
    if text.is_empty() {
        return;
    }
    let start = text.as_ptr() as usize;
    let end = start + text.len() - 1;
    let prev = GLOBAL_CODE
        .write()
        .unwrap()
        .insert(end, (start, code.clone()));
    assert!(prev.is_none());
}

/// Unregisters a code mmap from the global map.
///
/// Must have been previously registered with `register`.
pub fn unregister_code(code: &Arc<CodeMemory>) {
    let text = code.text();
    if text.is_empty() {
        return;
    }
    let end = (text.as_ptr() as usize) + text.len() - 1;
    let code = GLOBAL_CODE.write().unwrap().remove(&end);
    assert!(code.is_some());
}

#[test]
fn test_frame_info() -> Result<(), anyhow::Error> {
    use crate::*;
    let mut store = Store::<()>::default();
    let module = Module::new(
        store.engine(),
        r#"
            (module
                (func (export "add") (param $x i32) (param $y i32) (result i32) (i32.add (local.get $x) (local.get $y)))
                (func (export "sub") (param $x i32) (param $y i32) (result i32) (i32.sub (local.get $x) (local.get $y)))
                (func (export "mul") (param $x i32) (param $y i32) (result i32) (i32.mul (local.get $x) (local.get $y)))
                (func (export "div_s") (param $x i32) (param $y i32) (result i32) (i32.div_s (local.get $x) (local.get $y)))
                (func (export "div_u") (param $x i32) (param $y i32) (result i32) (i32.div_u (local.get $x) (local.get $y)))
                (func (export "rem_s") (param $x i32) (param $y i32) (result i32) (i32.rem_s (local.get $x) (local.get $y)))
                (func (export "rem_u") (param $x i32) (param $y i32) (result i32) (i32.rem_u (local.get $x) (local.get $y)))
            )
         "#,
    )?;
    // Create an instance to ensure the frame information is registered.
    Instance::new(&mut store, &module, &[])?;

    for (i, alloc) in module.compiled_module().finished_functions() {
        let (start, end) = {
            let ptr = alloc.as_ptr();
            let len = alloc.len();
            (ptr as usize, ptr as usize + len)
        };
        for pc in start..end {
            let (frame, _) = store
                .as_context()
                .0
                .modules()
                .lookup_frame_info(pc)
                .unwrap();
            assert!(
                frame.func_index() == i.as_u32(),
                "lookup of {:#x} returned {}, expected {}",
                pc,
                frame.func_index(),
                i.as_u32()
            );
        }
    }
    Ok(())
}
