# Instrumentation (Q-i / Q-ii) — answered before the change

## Q-i: where the quoted BCI set is computed, and why the iinc statement is not in it

The quote is pushed by `Builder::fallback` (`crates/jarde-java/src/build.rs:24117`), whose `bcis`
argument is chosen by the **statement** that failed to render:

- old-value store (`SA.postSelf`, `SD.arrSelf`, `FS.compoundSelf`): the `Operation::Store` arm of
  the per-instruction statement walk (`build.rs:18583` → `18631/18634` → `render_value` → `20543`
  `Operation::Load` arm). On `Err` it calls `self.fallback(bcis, &reason, at)` with the **store's own
  BCI plus the load that fed it** (`// @bytecode 6 2` for `postSelf`, `// @bytecode 12 7 8` for
  `arrSelf`). The `iinc` at BCI 3/9 is a *different instruction* and is walked separately: its own
  `Operation::Increment` renders as `local0 = local0 + 1;` and succeeds, because nothing in that
  arm asks whether the slot's value was provable for a sibling instruction.
- copy (`BF.enable`, `PC.viaArg`): `render_value`'s `Operation::Duplicate` arm (`build.rs:20501`)
  fails; the swallowing statement is the compound RMW and the quote covers
  `// @bytecode 9 8 2` — the receiver `dup`, the `getfield`, the `ior` — while the `return;` /
  `return this.flags;` tail is a separate statement that renders fine on its own.
- dependency chain (`AD.add`, `P02_multianewarray`): `prepare_deferred_bindings`
  (`build.rs:17765`) records a `BindingRejection` keyed by the producer's anchor BCI and published
  in `binding_rejections` (`build.rs:8285`); `statement_at` (`build.rs:18517`) then calls
  `reject_binding` (`build.rs:18398`), which quotes `binding_quote_bcis(anchor)` — the producer's
  chain only, never the method's tail.
- multi-consumer (`NI`): same `BindingRejection` path, reason from `build.rs:17824`.
- bound receiver (`OP.sideEffect`) and the conversion-evidence family (`CP.byAnon`) fail at the
  *value/argument* layer (`lambda.rs:875`, `build.rs:21769`), so their statement is quoted while the
  sibling statements survive.

So the partition is per-statement and the failed-value set is never consulted across statements:
that is exactly the hole.

## Q-ii: what absorbing the sibling statements does to existing refusals

Two shapes exist and they are kept distinct, because the guard only ever *widens a quote*:

1. **Already-uncompilable text stays byte-identical.** `SA.postOther` survives as
   `int local0 = 5;` + a quote with no `return` → `javac` fails on "missing return statement".
   Widening is only applied when the stripped body **still compiles**; a body that already fails to
   compile is left exactly as it is (verified: `completes_normally` at `build.rs:7649` is the same
   oracle the existing control-flow-exit escalation uses at `build.rs:7550`). The "local crosses"
   refusals and every `no bounded final expression consumer` cascade are unchanged for the same
   reason: their text is not widened, so no diagnostic line moves.
2. **Compilable-wrong text is widened.** For `i = i++` the absorbing step replaces
   `local0 = local0 + 1;` and `return local0;` with quotes carrying the *same* family reason verbatim,
   so no new refusal code appears; where a statement-level effect was swallowed and no surviving
   statement names its provenance (`BF.enable`'s `return;`, `NI`'s print tail, `AD.add`), the whole
   method degrades to one quote, which is the spec's "整方法响亮拒绝" branch.

Void is the case the pre-audit left open: a fully-quoted `void` body renders as `{ }`, which
**compiles and silently no-ops** (root reproduced independently with a V/V2 probe: exit 0, `8` vs
`0`). Root ruled the presentation fix belongs to this change: a zero-statement presentation of a
`void` body carries `jarde_refused_body();` — a jarde-reserved undefined symbol, so the stripped text
fails `javac` with "cannot find symbol". Non-`void` zero-statement bodies are left byte-identical
(an empty body already fails the missing-return check).
