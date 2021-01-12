-module(mult).

-export([mult/2]).

%% @doc Multiplies 2 numbers.
%%
-spec mult(X :: number(), Y :: number()) -> number().
mult(X, Y) when is_number(X) and is_number(Y) -> X * Y.
