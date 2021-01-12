const impl = ctx => {
  const { name, srcs } = ctx.cfg();

  const transitiveDeps = ctx.transitiveDeps().flatMap(dep => dep.outs);

  const out = `${name}.tar.gz`;
  ctx.action().declareOutputs([ out ]);

  ctx.action().exec({
    cmd: "tar",
    args: ["czfh", out, ...srcs, ...transitiveDeps]
  });
};

export default Zap.Rule({
  name: "archive",
  mnemonic: "Tar",
  impl,
  cfg: {
    name: label(),
    deps: [label()],
    srcs: [file()],
  },
});
