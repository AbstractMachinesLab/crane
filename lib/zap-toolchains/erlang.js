import ErlangToolchain from "zap@toolchains/erlang";

const impl = ctx => {
  const root = ctx.archive().unpackedRoot();
  const { build_opts } = ctx.cfg();

  if ctx.archive().fromSources() {
    ctx.action().exec(root.join("otp_build"), ["all", ...build_opts]);
  }

  const binRoot = root.join("bin");
  ctx.declareOutputs([
    binRoot.join("erlc"),
    binRoot.join("erl"),
  ]);
};

export default Toolchain({
  impl,
  cfg: {
    otp_build_opts: [string()],
  }
});
