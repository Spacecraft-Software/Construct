# Spacecraft Elixir Guidelines — Full Reference

**Version:** 1.0
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the SKILL.md for cases requiring deeper OTP patterns,
supervision design, or testing detail. Load it when the user asks for "complete
guidelines", "supervision tree", "GenServer skeleton", or when reviewing a real
OTP application. All examples are hand-written illustrations of public idioms.

## 1. Supervision tree (the backbone)

Start every long-lived process from a supervisor with a deliberate strategy:

```elixir
defmodule MyApp.Application do
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Registry, keys: :unique, name: MyApp.Registry},
      {MyApp.Cache, []},                                   # a GenServer
      {DynamicSupervisor, strategy: :one_for_one, name: MyApp.WorkerSup}
    ]

    # :one_for_one  — restart only the crashed child (default)
    # :rest_for_one — restart the crashed child and everything started after it
    # :one_for_all  — restart all children together (tightly-coupled set)
    opts = [strategy: :one_for_one, name: MyApp.Supervisor, max_restarts: 3, max_seconds: 5]
    Supervisor.start_link(children, opts)
  end
end
```

`max_restarts`/`max_seconds` cap a crash loop: exceeding the budget kills the
supervisor and escalates upward, instead of spinning a doomed child forever.

## 2. GenServer skeleton (serialized state)

```elixir
defmodule MyApp.Cache do
  use GenServer

  # ---- Client API (runs in the caller) ----
  def start_link(opts), do: GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  @spec fetch(term()) :: {:ok, term()} | :error
  def fetch(key), do: GenServer.call(__MODULE__, {:fetch, key})
  def put(key, value), do: GenServer.cast(__MODULE__, {:put, key, value})

  # ---- Server callbacks (run in the process) ----
  @impl true
  def init(_opts), do: {:ok, %{}}

  @impl true
  def handle_call({:fetch, key}, _from, state) do
    {:reply, Map.fetch(state, key), state}
  end

  @impl true
  def handle_cast({:put, key, value}, state) do
    {:noreply, Map.put(state, key, value)}
  end
end
```

Keep callbacks fast. For slow work, reply early and continue in a `Task` (or
return `{:noreply, state, {:continue, term}}` and handle it in `handle_continue/2`).

## 3. Bounded parallel fan-out

```elixir
@spec process_all([item()]) :: [result()]
def process_all(items) do
  items
  |> Task.async_stream(&process/1,
       max_concurrency: System.schedulers_online(),
       ordered: false,
       timeout: 30_000,
       on_timeout: :kill_task)
  |> Enum.flat_map(fn
       {:ok, result}      -> [result]
       {:exit, _reason}   -> []          # crashed/timed-out task: log + drop or retry
     end)
end
```

`max_concurrency` gives back-pressure; `on_timeout: :kill_task` stops a stuck
item from wedging the stream. Never replace this with an unbounded `Task.async`
loop.

## 4. `with` for multi-step happy paths

```elixir
def checkout(cart_id, payment) do
  with {:ok, cart}    <- Carts.fetch(cart_id),
       :ok            <- Carts.ensure_not_empty(cart),
       {:ok, charge}  <- Payments.charge(payment, cart.total),
       {:ok, order}   <- Orders.create(cart, charge) do
    {:ok, order}
  end
  # any {:error, reason} short-circuits and is returned as-is
end
```

## 5. ETS cache (shared, read-heavy)

```elixir
def init(_) do
  table = :ets.new(:my_cache, [:set, :named_table, :public, read_concurrency: true])
  {:ok, %{table: table}}
end

def get(key) do
  case :ets.lookup(:my_cache, key) do
    [{^key, value}] -> {:ok, value}
    []              -> :error
  end
end
```

Use `:persistent_term` instead when the data is effectively read-only and read
on extremely hot paths (config, compiled routing tables) — writes are expensive,
reads are free.

## 6. Typespecs + Dialyzer

```elixir
@type id :: pos_integer()
@type user :: %User{id: id(), email: String.t()}

@spec fetch_user(id()) :: {:ok, user()} | {:error, :not_found}
def fetch_user(id) when is_integer(id) and id > 0 do
  # ...
end
```

Add `:dialyxir` to deps and run `mix dialyzer` in CI; treat warnings as failures.

## 7. Testing with ExUnit

```elixir
defmodule MyApp.CacheTest do
  use ExUnit.Case, async: true       # async when the test shares no global state

  describe "fetch/1" do
    test "returns {:ok, value} for a stored key" do
      start_supervised!(MyApp.Cache)
      MyApp.Cache.put(:k, 42)
      assert {:ok, 42} == MyApp.Cache.fetch(:k)
    end

    test "returns :error for a missing key" do
      start_supervised!(MyApp.Cache)
      assert :error == MyApp.Cache.fetch(:absent)
    end
  end
end
```

- `start_supervised!/1` ties process lifetime to the test, preventing leaks.
- One behaviour per test; assert `expected == actual` (the formatter/Credo will
  flag ordering); use `ExUnit.CaptureLog` for log assertions.
- Property tests with `StreamData` for invariants:
  ```elixir
  use ExUnitProperties
  property "encode |> decode is identity" do
    check all term <- term() do
      assert decode(encode(term)) == term
    end
  end
  ```

## 8. Common pitfalls & fixes

| Pitfall                         | Symptom                          | Fix                                                |
|---------------------------------|----------------------------------|----------------------------------------------------|
| Pure logic in a GenServer       | Serialized bottleneck            | Make it a plain module of functions                |
| Unbounded `Task.async` loop     | Mailbox/OOM blow-up              | `Task.async_stream` with `max_concurrency`         |
| `String.to_atom/1` on input     | Atom-table exhaustion → node death | `String.to_existing_atom/1`                       |
| Slow work in `handle_call`      | Process unresponsive             | Offload to a `Task`; reply early / `handle_continue`|
| Catch-all `try/rescue`          | Faults hidden, no recovery       | Let it crash; supervise; rescue only expected cases |
| Big term passed repeatedly      | CPU on message copying           | ETS / keep data process-local                       |

## 9. Code-review mandate

Any OTP change must pass:
1. `mix format --check-formatted && mix compile --warnings-as-errors`
2. `mix credo --strict && mix dialyzer`
3. `mix test --cover` (async where safe)
4. Manual review of supervision strategy, restart limits, and message protocol

**This skill ensures every Elixir process written at Spacecraft Software is
supervised, fault-tolerant, and scales across all BEAM schedulers.**
