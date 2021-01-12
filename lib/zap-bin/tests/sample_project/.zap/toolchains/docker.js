const impl = ctx => {
  const root = ctx.archive().unpackedRoot();
  const binRoot = root.join("bin");
  const DENO = binRoot.join("docker");
  ctx.provides({ DENO });
};

export default Zap.Toolchain({
  name: "docker",
  mnemonic: "Docker",
  impl,
});

