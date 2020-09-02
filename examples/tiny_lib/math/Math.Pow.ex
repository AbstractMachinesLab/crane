defmodule Math.Pow do
  @moduledoc false

  @doc """
  Elevate `n` to the power `e`.
  """
  @spec pow(n :: number(), e :: number()) :: number()
  def pow(n, e) when is_number(n) and is_number(e), do: :math.pow(n, e)

end
