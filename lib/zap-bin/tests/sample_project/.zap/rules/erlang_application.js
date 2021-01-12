import ErlangToolchain, {HEADER_EXT, BEAM_EXT, ERL_EXT, APP_EXT, APP_SRC_EXT} from "../toolchains/erlang.js";

const impl = ctx => {
  const { name, label, config } = ctx.cfg();

  const ebin = Label.path(label).join(`ebin`);

  const app_file = ebin.join(File.filename(config).replace(APP_SRC_EXT, APP_EXT));
  ctx.action().declareOutputs([ app_file ]);
  ctx.action().copy({ src: config, dst: app_file });

  const transitiveDeps = ctx.transitiveDeps().flatMap(dep => dep.outs);
  transitiveDeps
    .filter(path => path.endsWith(BEAM_EXT) || path.endsWith(APP_EXT))
    .forEach( beam => {
      const src = beam;
      const dst = ebin.join(File.filename(beam));
      ctx.action().declareOutputs([ dst ]);
      ctx.action().copy({ src, dst });
    });

};

export default Zap.Rule({
  name: "erlang_application",
  mnemonic: "ErlApp",
  impl,
  cfg: {
    name: label(),
    config: file(),
    deps: [label()],
  },
  toolchains: [ErlangToolchain]
});
