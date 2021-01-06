

Alrigth, let me know if this makes sense to you.

## how zap works

note: this is not explaining the sandboxing/caching

You've got a **workspace**, that would be your monorepo. There you have a
`Workspace.toml` file.

Your monorepo has a bunch of **packages** in it, some are libraries, some are
binaries, some are JS bundles. Each **package** is essentially a `Build.toml`
file.

Example:

```
<workspace-root>
├── common
│   └── models
│       └── Build.toml
├── frontend
│   └── Build.toml
├── server
│   ├── db
│   │   └── Build.toml
│   ├── http
│   │   └── Build.toml
│   └── workers
│       └── Build.toml
└── Workspace.toml
```

Every `Build.toml` file declares one or more **targets**.

A **target** represents a compilation unit in your workspace. They are defined
by at least 2 things: a **rule**, that defines how it can be configured and how
it will be built; and a **label**, which will be the global identifier of that
target in the entire workspace.

Example:

```toml
# file: //common/models/Build.toml

[[ocaml_library]]
name = "ml"
srcs = [ "*.ml", "*.mli" ]
deps = []

[[jsoo_library]]
name = "js"
srcs = [ "index.js" ]
deps = [ ":ml "]

```

Here the first **rule** is `ocaml_library` and the **label** is a composite of
the path to the package and the name of the rule: `//common/models:ml`.

The second rule is `jsoo_library`, and its label is `//common/models:js`.

A **rule** specifies how to produce some outputs based on its input
configuration. Executing a rule produce a series of **actions** that, when
carried out, will generate the rule's outputs.

Every rule is right now _hard coded_ into the build system, but it should be
possible to expose a small language to write them on an ad-hoc basis.
