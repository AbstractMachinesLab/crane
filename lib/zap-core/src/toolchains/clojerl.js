const impl = ctx => {
  const root = ctx.archive().unpackedRoot();
  const binRoot = root.join("bin");
  const CLOJERL = binRoot.join("clojerl");
  ctx.provides({ CLOJERL });
};

export default Zap.Toolchain({
  name: "clojerl",
  mnemonic: "ClojErl",
  impl,
});
