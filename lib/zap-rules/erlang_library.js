import ErlangToolchain from "zap@toolchains/erlang";

const impl = ctx => {
  const { name, deps, srcs, headers, behaviors } = ctx.cfg();

  const transitiveDeps = ctx.transitiveDeps().flatMap(|dep| dep.outs());
  const includePaths = transitiveDeps
    .filter(path => path.endsWith(ErlangToolchain.HEADER_EXT))
    .concat(headers)
    .flatMap(path => ["-I", file_parent(path)])
    .unique();

  const extraLibPaths = transitiveDeps
    .filter(path => path.endsWith(ErlangToolchain.BEAM_EXT))
    .flatMap(path => ["-pa", file_parent(path)])
    .unique();

  const finalSrcs = transitiveDeps
    .filter(path => path.endsWith(ErlangToolchain.ERL_EXT))
    .concat(behaviors)
    .concat(srcs);

  const outputs = behaviors
    .concat(srcs)
    .map(|erl| file_replace_extension(erl, ErlangToolchain.BEAM_EXT))
    .concat(headers);

  ctx.action().declareOutputs(outputs);

  const paths = {};
  srcs.forEach( src => {
    const parent = file_parent(src);
    paths[parent] = paths[parent] || [];
    paths[parent].push(src);
  });

  paths.entries().forEach( ([out, srcs]) => {
    const args = [
      "-b", "beam", "-Wall",
      ...includePaths,
      ...extraLibPaths,
      "-o", out, "--",
      ...srcs
    ];

    ctx.action().exec(ErlangToolchain.ERLC, args);
  });
};

export default Zap.Rule({
  impl,
  cfg: {
    name: label(),
    deps: [label()],
    srcs: [file()],
    headers: [file()],
    behaviors: [file()],
  }
  toolchains: [ErlangToolchain]
});
