-module(math).

-export([test/0]).

test() ->
  A = mult:mult(2, 3), % from an erlang lib
  B = 'div':add(A),    % from a clojerl lib
  C = add:add(B, B),   % from a gleam lib
  'Elixir.Math.Pow':pow(2, C). % from an elixir lib
