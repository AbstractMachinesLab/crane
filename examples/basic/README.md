# Simplest Zap Example

Just a tiny example.

You can run `zap run //:shell`

```erlang
examples/basic λ zap run //:shell
10:03:51 INFO :: Workspace: basic
10:03:51 INFO :: Target: //:shell
10:03:51 INFO :: Planning build...
10:03:51 INFO :: Readying toolchains: [Erlang]
10:03:51 INFO :: Building target and dependencies: //:shell
10:03:51 INFO :: Built 1 artifacts in 176ms
10:03:51 INFO :: Running target:
Eshell V11.0.3  (abort with ^G)
hello_world:run().
1> Hello, world!ok
```
