const impl = ctx => {
  const root = ctx.archive().unpackedRoot();
  const binRoot = root.join("bin");
  const DENO = binRoot.join("deno");
  ctx.provides({ DENO });
};

export default Zap.Toolchain({
  name: "deno",
  mnemonic: "Deno",
  impl,
});
