pub const TOOLCHAINS: [(&'static str, &'static str); 4] = [
    (
        "https://zap.build/toolchains/archive",
        include_str!("archive.js"),
    ),
    ("https://zap.build/toolchains/deno", include_str!("deno.js")),
    (
        "https://zap.build/toolchains/elixir",
        include_str!("elixir.js"),
    ),
    (
        "https://zap.build/toolchains/erlang",
        include_str!("erlang.js"),
    ),
];
