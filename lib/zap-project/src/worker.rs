use super::*;
use anyhow::*;
use dashmap::DashMap;
use log::*;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use zap_buildscript::*;
use zap_core::*;

pub struct ZapWorker {
    pub bs_ctx: BuildScript,
    pub config: ZapConfig,
    pub dep_graph: DepGraph,
    pub rule_manager: Arc<RwLock<RuleManager>>,
    pub toolchain_manager: Arc<RwLock<ToolchainManager>>,
    pub workspace: Workspace,

    pub action_map: Arc<DashMap<Label, Vec<Action>>>,
    pub output_map: Arc<DashMap<Label, Vec<PathBuf>>>,
}

impl ZapWorker {
    pub fn new(config: ZapConfig) -> Result<ZapWorker, anyhow::Error> {
        Ok(ZapWorker {
            config,
            workspace: Workspace::default(),
            rule_manager: Arc::new(RwLock::new(RuleManager::default())),
            toolchain_manager: Arc::new(RwLock::new(ToolchainManager::default())),
            dep_graph: DepGraph::default(),
            bs_ctx: BuildScript::new()?,

            action_map: Arc::new(DashMap::new()),
            output_map: Arc::new(DashMap::new()),
        })
    }

    pub async fn load(&mut self, root: &PathBuf) -> Result<(), anyhow::Error> {
        self.scan(&root)?;
        self.configure_bs_ctx()?;
        self.load_default_toolchains().await?;
        self.load_local_toolchains().await?;
        self.load_default_rules().await?;
        self.load_local_rules().await?;
        self.create_dep_graph()?;
        Ok(())
    }

    pub fn config(&self) -> &ZapConfig {
        &self.config
    }

    pub fn toolchain_manager(&self) -> Arc<RwLock<ToolchainManager>> {
        self.toolchain_manager.clone()
    }

    pub fn rule_manager(&self) -> Arc<RwLock<RuleManager>> {
        self.rule_manager.clone()
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn dep_graph(&self) -> &DepGraph {
        &self.dep_graph
    }

    pub fn scan(&mut self, root: &PathBuf) -> Result<&mut ZapWorker, anyhow::Error> {
        self.workspace = WorkspaceScanner::scan(root, &*self.toolchain_manager.read().unwrap())
            .context("Could not create a workspace.")?;
        Ok(self)
    }

    async fn load_default_toolchains(&mut self) -> Result<(), anyhow::Error> {
        let mgr = (*self.toolchain_manager).read().unwrap();
        for (name, src) in zap_toolchains::TOOLCHAINS.iter() {
            mgr.load_from_str(&name, &src, &mut self.bs_ctx).await?;
        }
        Ok(())
    }

    async fn load_local_toolchains(&mut self) -> Result<(), anyhow::Error> {
        (*self.toolchain_manager)
            .read()
            .unwrap()
            .load_from_workspace(&self.workspace, &mut self.bs_ctx)
            .await
    }

    async fn load_default_rules(&mut self) -> Result<(), anyhow::Error> {
        let mgr = (*self.rule_manager).read().unwrap();
        for (name, src) in zap_rules::RULES.iter() {
            mgr.load_from_str(&name, &src, &mut self.bs_ctx).await?;
        }
        Ok(())
    }

    async fn load_local_rules(&mut self) -> Result<(), anyhow::Error> {
        (*self.rule_manager)
            .read()
            .unwrap()
            .load_from_workspace(&self.workspace, &mut self.bs_ctx)
            .await
    }

    fn configure_bs_ctx(&mut self) -> Result<(), anyhow::Error> {
        self.bs_ctx.runtime.register_op(
            "console.log",
            deno_core::json_op_sync(|_state, json, _zero_copy| {
                println!("{}", json.to_string());
                Ok(Value::from(""))
            }),
        );

        self.bs_ctx.runtime.register_op(
            "File.parent",
            deno_core::json_op_sync(|_state, json, _zero_copy| {
                let file_path = json.as_str().unwrap();
                let path = PathBuf::from(file_path);
                let parent = path.parent().unwrap().to_str().unwrap();
                Ok(Value::from(parent))
            }),
        );

        self.bs_ctx.runtime.register_op(
            "File.withExtension",
            deno_core::json_op_sync(|_state, json, _zero_copy| {
                let obj = json.as_object().unwrap();
                trace!("File.withExtension({:?})", obj);
                let file_path = obj["path"].as_str().unwrap();
                let path = PathBuf::from(file_path);
                let ext = obj["ext"].as_str().unwrap();

                let final_path = path.with_extension(ext.strip_prefix(".").unwrap());
                Ok(Value::from(final_path.to_str().unwrap()))
            }),
        );

        let output_map = self.output_map.clone();
        self.bs_ctx.runtime.register_op(
            "Zap.Targets.compute::ctx.actions.declareOutputs",
            deno_core::json_op_sync(move |_state, json, _zero_copy| {
                let obj = json.as_object().unwrap();
                let label: Label = obj["label"].as_str().unwrap().into();
                let outs: Vec<PathBuf> = obj["outs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| PathBuf::from(o.as_str().unwrap()))
                    .collect();
                trace!(
                    "Zap.Targets.compute::ctx.actions.declareOutputs({}, {:?})",
                    label.to_string(),
                    outs
                );
                match output_map.get(&label) {
                    None => output_map.insert(label, outs),
                    Some(entry) => {
                        let last_outs = entry.value();
                        let mut new_outs = vec![];
                        new_outs.extend_from_slice(&last_outs);
                        new_outs.extend_from_slice(&outs);
                        debug!("Updating output_map: {:?}", &new_outs);
                        output_map.insert(label, new_outs)
                    }
                };
                Ok(Value::from(""))
            }),
        );

        let action_map = self.action_map.clone();
        self.bs_ctx.runtime.register_op(
            "Zap.Targets.compute::ctx.actions.exec",
            deno_core::json_op_sync(move |_state, json, _zero_copy| {
                let obj = json.as_object().unwrap();
                let label: Label = obj["label"].as_str().unwrap().into();
                let cmd: PathBuf = PathBuf::from(obj["cmd"].as_str().unwrap());
                let args: Vec<&str> = obj["args"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| a.as_str().unwrap())
                    .collect();
                trace!(
                    "Zap.Targets.compute::ctx.actions.exec({}, {:?}, {:?})",
                    label.to_string(),
                    cmd,
                    args
                );

                let cwd: Option<PathBuf> = obj
                    .get("cwd")
                    .and_then(|val| val.as_str())
                    .map(|cwd| PathBuf::from(cwd.to_string()));

                let mut action = Action::exec(cmd);
                action.args(&args);
                if let Some(cwd) = cwd {
                    action.cwd(&cwd);
                }
                let action = action.build();

                match action_map.get(&label) {
                    None => action_map.insert(label, vec![action]),
                    Some(entry) => {
                        let last_actions = entry.value();
                        let mut new_actions = vec![];
                        new_actions.extend_from_slice(&last_actions);
                        new_actions.extend_from_slice(&[action]);
                        debug!("Updating action_map: {:?}", &new_actions);
                        action_map.insert(label, new_actions)
                    }
                };
                Ok(Value::from(""))
            }),
        );

        let toolchain_manager = self.toolchain_manager.clone();
        let cache_root = self.config.cache_root.clone();
        self.bs_ctx.runtime.register_op(
            "Zap.Toolchain",
            deno_core::json_op_sync(move |_state, json, _zero_copy| {
                let rule = ZapWorker::rule_from_json(json)?;

                trace!("Registering toolchain: {}", &rule.name().to_string());
                (*toolchain_manager)
                    .read()
                    .unwrap()
                    .register_toolchain(rule, cache_root.clone());

                Ok(Value::from(""))
            }),
        );

        let rule_manager = self.rule_manager.clone();
        self.bs_ctx.runtime.register_op(
            "Zap.Rule",
            deno_core::json_op_sync(move |_state, json, _zero_copy| {
                let rule = ZapWorker::rule_from_json(json)?;

                trace!("Registering rule: {}", &rule.name().to_string());
                (*rule_manager).read().unwrap().register(rule);

                Ok(Value::from(""))
            }),
        );

        self.bs_ctx
            .runtime
            .execute("<prelude>", include_str!("prelude.js"))?;

        Ok(())
    }

    fn create_dep_graph(&mut self) -> Result<(), anyhow::Error> {
        WorkspaceScanner::collect_targets(
            &mut self.workspace,
            &(*self.rule_manager).read().unwrap(),
        )?;
        let mut targets = self.workspace.targets().to_vec();
        targets.extend((*self.toolchain_manager).read().unwrap().targets());
        self.dep_graph = DepGraph::from_targets(&targets)?;
        Ok(())
    }

    fn rule_from_json(json: serde_json::Value) -> Result<Rule, anyhow::Error> {
        let rule_spec = json.as_object().context(format!(
            "Expeced RuleSpec to be an Object, instead found: {:?}",
            json
        ))?;

        let json_cfg = &rule_spec["cfg"].as_object().context(format!(
            "Expected RuleSpec 'cfg' key to be an Object, instead found: {:?}",
            &rule_spec["cfg"]
        ))?;

        let mut cfg = HashMap::new();
        for (k, t) in json_cfg.iter() {
            let value_type = match t.as_str().unwrap() {
                        "label" => Ok(CfgValueType::Label),
                        "file" => Ok(CfgValueType::File),
                        "string" => Ok(CfgValueType::String),
                        "list_of_label" => Ok(CfgValueType::List(Box::new(CfgValueType::Label))),
                        "list_of_file" => Ok(CfgValueType::List(Box::new(CfgValueType::File))),
                        "list_of_string" => Ok(CfgValueType::List(Box::new(CfgValueType::String))),
                        _ => Err(anyhow!("Unrecognized rule config key type {} -- valid types are  label(), file(), string(), and their array variants", t.to_string())),
                    }?;

            cfg.insert(k.to_string(), value_type);
        }
        let config = ConfigSpec(cfg);

        let default_cfg = DashMap::new();
        for (k, v) in rule_spec["defaults"]
            .as_object()
            .context("Expected 'defaults' to be an Object".to_string())?
            .iter()
        {
            let t = config
                .get(k)
                .context(format!("Could not find type for key {:?}", k))?;
            let typed_value = (v.clone(), t.clone()).into();
            default_cfg.insert(k.to_string(), typed_value);
        }
        let defaults = RuleConfig(default_cfg);

        let toolchains: Vec<Label> = (&rule_spec["toolchains"])
            .as_array()
            .context(format!(
                "Expected RuleSpec 'toolchains' key to be an Array, instead found: {:?}",
                &rule_spec["toolchains"]
            ))?
            .iter()
            .map(|t| {
                let string = t.as_str().unwrap();
                Label::new(string)
            })
            .collect();

        let rule = Rule::new(
            rule_spec["name"].as_str().unwrap().to_string(),
            rule_spec["mnemonic"].as_str().unwrap().to_string(),
            toolchains,
            config,
            defaults,
        );

        Ok(rule)
    }
}
