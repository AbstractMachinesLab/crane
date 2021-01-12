export const HEADER_EXT = ".hrl";
export const BEAM_EXT = ".beam";
export const ERL_EXT = ".erl";
export const APP_EXT = ".app";
export const APP_SRC_EXT = ".app.src";

const impl = ctx => {
  const {
    archiveKind,
    archiveSha1,
    unarchivedRoot,
  } = ctx.cfg();

  if (archiveKind === "source") {
    ctx.action().exec({
      cmd: unarchivedRoot.join("otp_build"),
      args: ["all"],
      cwd: unarchivedRoot
    });
  }

  const binRoot = unarchivedRoot.join("bin");
  const ERLC = binRoot.join("erlc");
  const ERL = binRoot.join("erl");
  ctx.provides({ ERLC, ERL });
  ctx.action().declareOutputs([]);
};

export default Zap.Toolchain({
  name: "//zap.build/toolchains:erlang",
  mnemonic: "Erlang/OTP",
  impl,
});
