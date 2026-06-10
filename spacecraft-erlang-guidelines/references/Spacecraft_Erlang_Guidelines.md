# Spacecraft Erlang Guidelines — Full Reference

**Version:** 1.0
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the SKILL.md for cases requiring deeper OTP patterns,
supervision design, or testing detail. Load it when the user asks for "complete
guidelines", "supervisor", "gen_server / gen_statem skeleton", or when reviewing
a real OTP application. All examples are hand-written illustrations of public
idioms.

## 1. Supervisor (the backbone)

```erlang
-module(myapp_sup).
-behaviour(supervisor).
-export([start_link/0, init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    %% one_for_one  — restart only the crashed child (default)
    %% rest_for_one — restart the crashed child and everything started after it
    %% one_for_all  — restart all children together (tightly-coupled set)
    SupFlags = #{strategy => one_for_one, intensity => 3, period => 5},
    Children = [
        #{id       => myapp_cache,
          start    => {myapp_cache, start_link, []},
          restart  => permanent,     %% permanent | transient | temporary
          shutdown => 5000,
          type     => worker,
          modules  => [myapp_cache]}
    ],
    {ok, {SupFlags, Children}}.
```

`intensity`/`period` cap a crash loop: more than `intensity` restarts within
`period` seconds kills the supervisor and escalates upward.

## 2. gen_server skeleton (serialized state)

```erlang
-module(myapp_cache).
-behaviour(gen_server).

%% API
-export([start_link/0, fetch/1, put/2]).
%% callbacks
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2]).

-spec start_link() -> {ok, pid()}.
start_link() -> gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

-spec fetch(term()) -> {ok, term()} | error.
fetch(Key) -> gen_server:call(?MODULE, {fetch, Key}).

put(Key, Value) -> gen_server:cast(?MODULE, {put, Key, Value}).

init([]) -> {ok, #{}}.

handle_call({fetch, Key}, _From, State) ->
    {reply, maps:find(Key, State), State}.

handle_cast({put, Key, Value}, State) ->
    {noreply, State#{Key => Value}}.

handle_info(_Msg, State) -> {noreply, State}.
terminate(_Reason, _State) -> ok.
```

Keep callbacks fast. Offload slow work to a separate (supervised) worker process
so the mailbox never blocks.

## 3. gen_statem skeleton (protocol / state machine)

```erlang
-module(conn_fsm).
-behaviour(gen_statem).
-export([start_link/0, callback_mode/0, init/1]).
-export([disconnected/3, connected/3]).

start_link() -> gen_statem:start_link(?MODULE, [], []).
callback_mode() -> state_functions.
init([]) -> {ok, disconnected, #{retries => 0}}.

disconnected({call, From}, connect, Data) ->
    %% attempt connect; on success move to `connected`
    {next_state, connected, Data, [{reply, From, ok}]};
disconnected(state_timeout, retry, Data) ->
    {keep_state, Data#{retries := maps:get(retries, Data) + 1}}.

connected(info, {tcp_closed, _Sock}, Data) ->
    %% drop back and schedule a retry via a state timeout
    {next_state, disconnected, Data, [{state_timeout, 1000, retry}]}.
```

State timeouts model retry/backoff without ad-hoc timer bookkeeping.

## 4. Bounded concurrency (supervised pool)

Do not `spawn` per request. Run a fixed pool of supervised workers and hand work
to a free one (a `poolboy`-style checkout, or a `simple_one_for_one`/dynamic
child pattern). Sketch:

```erlang
%% Borrow a worker, do the work, always return it — even on crash.
with_worker(Fun) ->
    Worker = poolboy:checkout(my_pool),
    try Fun(Worker)
    after poolboy:checkin(my_pool, Worker)
    end.
```

The pool size is your back-pressure knob; the supervisor restarts a crashed
worker without losing the pool.

## 5. ETS cache (shared, read-heavy)

```erlang
init([]) ->
    Tab = ets:new(my_cache, [set, named_table, public, {read_concurrency, true}]),
    {ok, #{tab => Tab}}.

get(Key) ->
    case ets:lookup(my_cache, Key) of
        [{Key, Value}] -> {ok, Value};
        []             -> error
    end.
```

Use `persistent_term` instead for effectively read-only, hot-path data (config,
routing tables): reads are free, writes are global and expensive.

## 6. Typespecs + Dialyzer

```erlang
-type id() :: pos_integer().
-type user() :: #{id := id(), email := binary()}.

-spec fetch_user(id()) -> {ok, user()} | {error, not_found}.
fetch_user(Id) when is_integer(Id), Id > 0 ->
    %% ...
    {error, not_found}.
```

Build a PLT once, then run `rebar3 dialyzer` in CI; treat warnings as failures.
Run `rebar3 xref` to catch calls to undefined/deprecated functions.

## 7. Testing: EUnit + Common Test

EUnit (fast unit tests, in-module or `_tests.erl`):

```erlang
-include_lib("eunit/include/eunit.hrl").

fetch_missing_returns_error_test() ->
    {ok, _Pid} = myapp_cache:start_link(),
    ?assertEqual(error, myapp_cache:fetch(absent)).
```

Common Test (integration, stateful, multi-node) — use for anything touching
supervision, distribution, or external systems:

```erlang
-module(cache_SUITE).
-include_lib("common_test/include/ct.hrl").
-export([all/0, init_per_testcase/2, end_per_testcase/2, put_then_fetch/1]).

all() -> [put_then_fetch].

init_per_testcase(_Case, Config) ->
    {ok, Pid} = myapp_cache:start_link(),
    [{pid, Pid} | Config].

end_per_testcase(_Case, _Config) -> ok.

put_then_fetch(_Config) ->
    ok = myapp_cache:put(k, 42),
    {ok, 42} = myapp_cache:fetch(k).
```

Property tests with `PropEr` for invariants (e.g. `decode(encode(T)) =:= T`).

## 8. Common pitfalls & fixes

| Pitfall                          | Symptom                          | Fix                                                 |
|----------------------------------|----------------------------------|-----------------------------------------------------|
| Hand-rolled `receive` loop       | Subtle shutdown/edge bugs        | Use `gen_server` / `gen_statem`                     |
| Pure logic inside a process      | Serialized bottleneck            | Plain module of functions                           |
| Unbounded `spawn` per request    | Mailbox/OOM blow-up              | Supervised, bounded worker pool                     |
| `binary_to_atom/2` on input      | Atom-table exhaustion → node death | `binary_to_existing_atom/2` / `list_to_existing_atom/1` |
| Slow work in `handle_call`       | Process unresponsive             | Offload to a worker; reply early                    |
| Catch-all `try/catch`            | Faults hidden, no recovery       | Let it crash; supervise; catch only expected cases  |
| Big term passed repeatedly       | CPU on message copying           | ETS / keep data process-local                       |

## 9. Code-review mandate

Any OTP change must pass:
1. `rebar3 compile` (`warnings_as_errors`) + `erlfmt`/`elvis`
2. `rebar3 dialyzer && rebar3 xref`
3. `rebar3 eunit && rebar3 ct` with coverage
4. Manual review of supervision strategy, restart intensity, and message protocol

**This skill ensures every Erlang process written at Spacecraft Software is
supervised, fault-tolerant, and scales across all BEAM schedulers.**
