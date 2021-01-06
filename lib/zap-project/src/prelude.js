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
const __TOOLCHAINS = {};

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



Zap.Toolchains = {};

Zap.Toolchains.exists = name => __TOOLCHAINS[name] !== null && __TOOLCHAINS[name] !== undefined;

Zap.Toolchains.register = (name, spec) => {
  __TOOLCHAINS[name] = spec;
};

Zap.Toolchains.getByName = name => {
    if (Zap.Toolchains.exists(name)) return __TOOLCHAINS[name];
    err(`Expected toolchain ${name} but could not find it in store!`);
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

  const ctx = {
    cfg: () => config,
    transitiveDeps: () => target.transitiveDeps,

    action: () => ({
      declareOutputs: outs => ffi("Zap.Targets.compute::ctx.actions.declareOutputs", {label, outs}),
      exec: (cmd, args) => ffi("Zap.Targets.compute::ctx.actions.exec", {label, cmd, args}),
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
  if (mnemonic.length > 6) err(`Rule mnemonic "${mnemonic}" should be shorter than 6 characters long`);

  const impl = spec.impl;
  if (!impl) err(`Rule ${name} must have an implementation.`);
  if (typeof impl !== "function") err(`Rule ${name} implementation should be a function, instead found: ${typeof impl}`);

  const cfg = spec.cfg;
  if (!cfg) err(`Rule ${name} must define a config map with   cfg   `);
  if (Object.entries(cfg).length == 0) err(`Config map for rule ${name} is empty! Try adding a   name: label()   key?`);
  spec.cfg = check_config(spec.cfg);

  if (Zap.Rules.exists(name)) err(`There already exists rule toolchain called ${name}, consider renaming yours`);
  Zap.Rules.register(name, spec);

  return ffi("Zap.Rule", spec);
};

Zap.Toolchain = spec => {
  const name = spec.name;
  if (!name) err(`Toolchain must have a string name.`);
  if (typeof name !== "string") err(`Toolchain name must be a string, instead found: ${name}.`);
  if (name === "") err(`Toolchain name was empty! Here's some inspiration:   super_lang  `);

  if (Zap.Toolchains.exists(name)) err(`There already exists a toolchain called ${name}, consider renaming yours`);
  Zap.Toolchains.register(name, spec);

  return ffi("Zap.Toolchain", spec);
};


/*******************************************************************************
 *
 * File APIs
 *
 ******************************************************************************/

const File = {};

File.parent = (path) => ffi("File.parent", path);
File.withExtension = (path, ext) => ffi("File.withExtension", {path, ext});
