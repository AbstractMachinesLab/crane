import ErlangToolchain, {HEADER_EXT, BEAM_EXT, ERL_EXT, APP_EXT, APP_SRC_EXT} from "../toolchains/erlang.js";


const impl = ctx => {
  const { name, label, config } = ctx.cfg();
  const deps = ctx.transitiveDeps();

  const root = Label.path(label).join(`release-${name}`);

  const version = "0.0.0";

  // NOTE(@ostera): traditionally OTP releases allow for multiple
  // different releases to coexist, to support hot code upgrades and downgrades
  const relRoot = root.join("releases").join(version);

  const config_file = relRoot.join(File.filename(config));
  ctx.action().declareOutputs([ config_file ]);
  ctx.action().copy({ src: config, dst: config_file });

  ctx.action().writeFile({
    data: `% Release generated with Zap.
{release,
  {${name}, ${version}},
  {erts, ${ErlangToolchain.provides().ERTS_VERSION}},
  [${deps.map(dep => `{${dep.name}, "0.0.0"}`).join(",\n   ")}]}.
  `,
    dst: relRoot.join(`${name}.rel`)
  });

  deps.forEach( dep => {
    // NOTE(@ostera): traditionally OTP release allow for different library
    // versions to coexist to allow for hot code upgrades and downgrades
    const depRoot = root.join("lib").join(`${dep.name}-0.0.0`);

    dep.outs
      .filter(path => path.endsWith(BEAM_EXT) || path.endsWith(APP_EXT))
      .forEach( src => {
        const dst = depRoot.join("ebin").join(File.filename(src));
        ctx.action().declareOutputs([ dst ]);
        ctx.action().copy({ src, dst });
      });
  });
};

export default Zap.Rule({
  name: "otp_release",
  mnemonic: "OtpRel",
  impl,
  cfg: {
    name: label(),
    config: file(),
    deps: [label()],
  },
  toolchains: [ErlangToolchain]
});
