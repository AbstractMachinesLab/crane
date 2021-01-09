export const HEADER_EXT = ".hrl";
export const BEAM_EXT = ".beam";
export const ERL_EXT = ".erl";

const impl = ctx => {
  const {
    unarchivedRoot,
    fromSources,
  } = ctx.cfg();

  if (fromSources) {
    ctx.action().exec(unarchivedRoot.join("otp_build"), ["all"]);
  }

  const binRoot = unarchivedRoot.join("bin");
  const ERLC = binRoot.join("erlc");
  const ERL = binRoot.join("erl");
  ctx.provides({ ERLC, ERL });
  ctx.action().declareOutputs([ERLC, ERL]);
};

export default Zap.Toolchain({
  name: "erlang",
  mnemonic: "Erlang/OTP",
  impl,
});
