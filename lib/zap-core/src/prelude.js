Deno.core.ops();
Deno.core.registerErrorClass('Error', Error);

Array.prototype.unique = function() {
  var arr = [];
  for (var i = 0; i < this.length; i++) {
    if (arr.indexOf(this[i]) === -1) {
      arr.push(this[i]);
    }
  }
  return arr;
}

String.prototype.join = function(str) {
  return this + "/" + str;
}

const ffi = (name, args) => Deno.core.jsonOpSync(name, args);
const err = x => { throw new Error(x); };

const label = () => "label";
const string = () => "string";
const file = () => "file";

const console = {
  log: (...args) => ffi("console.log", args)
};

/*******************************************************************************
 * Global stores
 ******************************************************************************/
const __RULES = {};
const __PROVIDES = {};

/*******************************************************************************
 *
 * Zap's Rule and Toolchain API
 *
 ******************************************************************************/

const Zap = {};


Zap.Rules = {};

Zap.Rules.exists = name => __RULES[name] !== null && __RULES[name] !== undefined;

Zap.Rules.register = (name, spec) => {
  __RULES[name] = spec;
};

Zap.Rules.getByName = name => {
    if (Zap.Rules.exists(name)) return __RULES[name];
    err(`Expected rule ${name} but could not find it in store!`);
};


Zap.Targets = {};

Zap.Targets.compute = target => {
  const label = target.label;
  const rule = Zap.Rules.getByName(target.rule);

  const config = Object.fromEntries(Object.entries(rule.cfg)
    .map( ([k, type]) => {
      const value = target.cfg[k] === undefined ? rule.defaults[k] : target.cfg[k];
      if (value === undefined) err(`Expected target  ${target.label}  to have config key '${k}' (of type ${type}) but it was not present in your Build.toml, and it doesn't have a default value.`);
      return [k, value];
    }));

  config.label = label;

  const ctx = {
    cfg: () => config,
    transitiveDeps: () => target.transitiveDeps,

    provides: outs => {
      __PROVIDES[label] = outs;
    },

    action: () => ({
      declareOutputs: outs => ffi("Zap.Targets.compute::ctx.actions.declareOutputs", {label, outs}),
      exec: ({cmd, args, cwd}) => ffi("Zap.Targets.compute::ctx.actions.exec", {label, cmd, args, cwd}),
      copy: ({src, dst}) => ffi("Zap.Targets.compute::ctx.actions.copy", {label, src, dst}),
      writeFile: ({data, dst}) => ffi("Zap.Targets.compute::ctx.actions.writeFile", {label, data, dst}),
    }),
  };

  rule.impl(ctx)
};


const check_config = cfg => {
  return Object.fromEntries(Object.entries(cfg).map(([k, t]) => {
    if (Array.isArray(t)) {
      if (t.length == 0) err(`Cfg map for rule ${name} found an empty list. Did you mean to use   [label()]   ?`);
      return [k, `list_of_${t[0]}`];
    } else {
      return [k, t];
    }
  }));
};

Zap.Rule = spec => {
  const name = spec.name;
  if (!name) err(`Rule must have a string name`);
  if (typeof name !== "string") err(`Rule name must be a string, instead found: ${name}`);
  if (name.length < 5) err(`Rule name "${name}" should be at least 5 characters long`);

  const mnemonic = spec.mnemonic;
  if (!mnemonic) err(`Rule must have a string mnemonic`);
  if (typeof mnemonic !== "string") err(`Rule mnemonic must be a string, instead found: ${mnemonic}`);

  const impl = spec.impl;
  if (!impl) err(`Rule ${name} must have an implementation.`);
  if (typeof impl !== "function") err(`Rule ${name} implementation should be a function, instead found: ${typeof impl}`);

  const cfg = spec.cfg;
  if (!cfg) err(`Rule ${name} must define a config map with   cfg   `);
  if (Object.entries(cfg).length == 0) err(`Config map for rule ${name} is empty! Try adding a   name: label()   key?`);
  spec.cfg = check_config(spec.cfg);

  spec.toolchains = (spec.toolchains || []);
  spec.toolchains = spec.toolchains.map( toolchain => toolchain.name );
  spec.defaults = spec.defaults || {};

  // if (Zap.Rules.exists(name)) err(`There already exists rule toolchain called ${name}, consider renaming yours`);
  Zap.Rules.register(name, spec);

  ffi("Zap.Rule", spec);

  return spec;
};


// NOTE(@ostera): Toolchains are actually just Rules on this side, since we'll need
// to invoke them like any other Rule later on in `Zap.Target.compute`
//
Zap.Toolchain = spec => {
  const name = spec.name;
  if (!name) err(`Toolchain must have a string name.`);
  if (typeof name !== "string") err(`Toolchain name must be a string, instead found: ${name}.`);
  if (name === "") err(`Toolchain name was empty! Here's some inspiration:   super_lang  `);

  spec.cfg = {
    archiveKind: string(),
    archiveName: string(),
    archivePrefix: string(),
    archiveSha1: string(),
    archiveTag: string(),
    unarchivedRoot: string(),
    archiveUrl: string(),
  }

  spec.defaults = {
    archiveKind: "release",
    archiveName: spec.name,
    archivePrefix: "",
    archiveSha1: "",
    archiveTag: "",
    unarchivedRoot: "./",
    archiveUrl: "",
  }

  spec.toolchains = []

  __PROVIDES[name] = {};
  spec.provides = () => __PROVIDES[name];

  // if (Zap.Rules.exists(name)) err(`There already exists a toolchain called ${name}, consider renaming yours`);
  Zap.Rules.register(name, spec);

  ffi("Zap.Toolchain", spec);

  return __RULES[name];
};


/*******************************************************************************
 *
 * File APIs
 *
 ******************************************************************************/

const File = {};

File.parent = (path) => ffi("File.parent", path);
File.filename = (path) => ffi("File.filename", path);
File.withExtension = (path, ext) => ffi("File.withExtension", {path, ext});

const Label = {};

Label.path = (label) => ffi("Label.path", label);
