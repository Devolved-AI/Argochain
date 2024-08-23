//! Implementation of a "compiler builder" for cranelift
//!
//! This module contains the implementation of how Cranelift is configured, as
//! well as providing a function to return the default configuration to build.

use anyhow::Result;
use cranelift_codegen::{
    isa::{self, OwnedTargetIsa},
    CodegenResult,
};
use std::fmt;
use std::sync::Arc;
use wasmtime_cranelift_shared::isa_builder::IsaBuilder;
use wasmtime_environ::{CacheStore, CompilerBuilder, Setting};

struct Builder {
    inner: IsaBuilder<CodegenResult<OwnedTargetIsa>>,
    linkopts: LinkOptions,
    cache_store: Option<Arc<dyn CacheStore>>,
}

#[derive(Clone, Default)]
pub struct LinkOptions {
    /// A debug-only setting used to synthetically insert 0-byte padding between
    /// compiled functions to simulate huge compiled artifacts and exercise
    /// logic related to jump veneers.
    pub padding_between_functions: usize,

    /// A debug-only setting used to force inter-function calls in a wasm module
    /// to always go through "jump veneers" which are typically only generated
    /// when functions are very far from each other.
    pub force_jump_veneers: bool,
}

pub fn builder() -> Box<dyn CompilerBuilder> {
    Box::new(Builder {
        inner: IsaBuilder::new(|triple| isa::lookup(triple).map_err(|e| e.into())),
        linkopts: LinkOptions::default(),
        cache_store: None,
    })
}

impl CompilerBuilder for Builder {
    fn triple(&self) -> &target_lexicon::Triple {
        self.inner.triple()
    }

    fn target(&mut self, target: target_lexicon::Triple) -> Result<()> {
        self.inner.target(target)?;
        Ok(())
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        // Special wasmtime-cranelift-only settings first
        if name == "wasmtime_linkopt_padding_between_functions" {
            self.linkopts.padding_between_functions = value.parse()?;
            return Ok(());
        }
        if name == "wasmtime_linkopt_force_jump_veneer" {
            self.linkopts.force_jump_veneers = value.parse()?;
            return Ok(());
        }

        self.inner.set(name, value)
    }

    fn enable(&mut self, name: &str) -> Result<()> {
        self.inner.enable(name)
    }

    fn build(&self) -> Result<Box<dyn wasmtime_environ::Compiler>> {
        let isa = self.inner.build()?;
        Ok(Box::new(crate::compiler::Compiler::new(
            isa,
            self.cache_store.clone(),
            self.linkopts.clone(),
        )))
    }

    fn settings(&self) -> Vec<Setting> {
        self.inner.settings()
    }

    fn enable_incremental_compilation(
        &mut self,
        cache_store: Arc<dyn wasmtime_environ::CacheStore>,
    ) -> Result<()> {
        self.cache_store = Some(cache_store);
        Ok(())
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Builder")
            .field("shared_flags", &self.inner.shared_flags().to_string())
            .finish()
    }
}
