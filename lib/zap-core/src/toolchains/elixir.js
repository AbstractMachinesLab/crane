const impl = ctx => {
  const root = ctx.archive().unpackedRoot();
  const binRoot = root.join("bin");
  const ELIXIRC = binRoot.join("elixirc");
  const IEX = binRoot.join("iex");
  ctx.provides({ ELIXIRC, IEX });
};

export default Zap.Toolchain({
  name: "elixir",
  mnemonic: "ElixirLang",
  impl,
});
