use crate::component::func::HostFunc;
use crate::component::linker::{Definition, NameMap, Strings};
use crate::types::matching;
use crate::Module;
use anyhow::{anyhow, bail, Context, Result};
use std::sync::Arc;
use wasmtime_environ::component::{
    ComponentTypes, TypeComponentInstance, TypeDef, TypeFuncIndex, TypeModule,
};

pub struct TypeChecker<'a> {
    pub types: &'a Arc<ComponentTypes>,
    pub strings: &'a Strings,
}

impl TypeChecker<'_> {
    pub fn definition(&self, expected: &TypeDef, actual: Option<&Definition>) -> Result<()> {
        match *expected {
            TypeDef::Module(t) => match actual {
                Some(Definition::Module(actual)) => self.module(&self.types[t], actual),
                _ => bail!("expected module found {}", desc(actual)),
            },
            TypeDef::ComponentInstance(t) => match actual {
                Some(Definition::Instance(actual)) => self.instance(&self.types[t], Some(actual)),
                None => self.instance(&self.types[t], None),
                _ => bail!("expected instance found {}", desc(actual)),
            },
            TypeDef::ComponentFunc(t) => match actual {
                Some(Definition::Func(actual)) => self.func(t, actual),
                _ => bail!("expected func found {}", desc(actual)),
            },
            TypeDef::Component(_) => bail!("expected component found {}", desc(actual)),
            TypeDef::Interface(_) => bail!("expected type found {}", desc(actual)),

            // not possible for valid components to import
            TypeDef::CoreFunc(_) => unreachable!(),
        }
    }

    fn module(&self, expected: &TypeModule, actual: &Module) -> Result<()> {
        let actual_types = actual.types();
        let actual = actual.env_module();

        // Every export that is expected should be in the actual module we have
        for (name, expected) in expected.exports.iter() {
            let idx = actual
                .exports
                .get(name)
                .ok_or_else(|| anyhow!("module export `{name}` not defined"))?;
            let actual = actual.type_of(*idx);
            matching::entity_ty(expected, self.types.module_types(), &actual, actual_types)
                .with_context(|| format!("module export `{name}` has the wrong type"))?;
        }

        // Note the opposite order of checks here. Every import that the actual
        // module expects should be imported by the expected module since the
        // expected module has the set of items given to the actual module.
        // Additionally the "matches" check is inverted here.
        for (module, name, actual) in actual.imports() {
            // TODO: shouldn't need a `.to_string()` here ideally
            let expected = expected
                .imports
                .get(&(module.to_string(), name.to_string()))
                .ok_or_else(|| anyhow!("module import `{module}::{name}` not defined"))?;
            matching::entity_ty(&actual, actual_types, expected, self.types.module_types())
                .with_context(|| format!("module import `{module}::{name}` has the wrong type"))?;
        }
        Ok(())
    }

    fn instance(&self, expected: &TypeComponentInstance, actual: Option<&NameMap>) -> Result<()> {
        // Like modules, every export in the expected type must be present in
        // the actual type. It's ok, though, to have extra exports in the actual
        // type.
        for (name, (_url, expected)) in expected.exports.iter() {
            // Interface types may be exported from a component in order to give them a name, but
            // they don't have a definition in the sense that this search is interested in, so
            // ignore them.
            if let TypeDef::Interface(_) = expected {
                continue;
            }
            let actual = self
                .strings
                .lookup(name)
                .and_then(|name| actual?.get(&name));
            self.definition(expected, actual)
                .with_context(|| format!("instance export `{name}` has the wrong type"))?;
        }
        Ok(())
    }

    fn func(&self, expected: TypeFuncIndex, actual: &HostFunc) -> Result<()> {
        actual.typecheck(expected, self.types)
    }
}

fn desc(def: Option<&Definition>) -> &'static str {
    match def {
        Some(def) => def.desc(),
        None => "nothing",
    }
}

impl Definition {
    fn desc(&self) -> &'static str {
        match self {
            Definition::Module(_) => "module",
            Definition::Func(_) => "func",
            Definition::Instance(_) => "instance",
        }
    }
}
