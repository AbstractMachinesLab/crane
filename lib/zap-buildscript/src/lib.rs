use anyhow::*;
pub use deno_core;
use log::*;
use std::path::PathBuf;
use std::rc::Rc;

pub struct BuildScript {
    pub runtime: deno_core::JsRuntime,
}

impl BuildScript {
    pub fn new() -> Result<BuildScript, anyhow::Error> {
        let rt_options = deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            ..Default::default()
        };

        let runtime = deno_core::JsRuntime::new(rt_options);

        Ok(BuildScript { runtime })
    }

    pub async fn load_from_str(
        &mut self,
        module_name: &str,
        module_code: &str,
    ) -> Result<deno_core::ModuleId, anyhow::Error> {
        trace!("Loading module from string");
        let mod_specifier = deno_core::ModuleSpecifier::resolve_url(&module_name)?;
        let mod_id = self
            .runtime
            .load_module(&mod_specifier, Some(module_code.to_string()))
            .await?;
        self.runtime.mod_evaluate(mod_id).await?;
        Ok(mod_id)
    }

    pub async fn load(&mut self, module: PathBuf) -> Result<deno_core::ModuleId, anyhow::Error> {
        trace!("Loading module at {:?}", &module);
        let mod_specifier = deno_core::ModuleSpecifier::resolve_path(module.to_str().unwrap())?;
        let mod_id = self.runtime.load_module(&mod_specifier, None).await?;
        self.runtime.mod_evaluate(mod_id).await?;
        Ok(mod_id)
    }
}
