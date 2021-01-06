export const HEADER_EXT = ".hrl";
export const BEAM_EXT = ".beam";
export const ERL_EXT = ".erl";

const impl = ctx => {
  const root = ctx.archive().unpackedRoot();
  const { build_opts } = ctx.cfg();

  if (ctx.archive().fromSources()) {
    ctx.action().exec(root.join("otp_build"), ["all", ...build_opts]);
  }

  const binRoot = root.join("bin");
  ctx.declareOutputs({
    ERLC: binRoot.join("erlc"),
    ERL: binRoot.join("erl"),
  });
};

export default Zap.Toolchain({
  name: "erlang",
  impl,
  cfg: {
    otp_build_opts: ["string"],
  }
});
