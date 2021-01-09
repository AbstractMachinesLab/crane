const impl = ctx => {
  const root = ctx.cfg().unarchivedRoot;
  const binRoot = root.join("bin");
  const CARAMELC = binRoot.join("caramelc");
  ctx.provides({ CARAMELC });
  ctx.action().declareOutputs([CARAMELC]);
};

export default Zap.Toolchain({
  name: "caramel",
  mnemonic: "Caramel",
  impl,
});
