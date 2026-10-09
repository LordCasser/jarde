# Frozen baseline family classification

This is the accepted-CLI baseline replay, not a candidate acceptance run. The frozen manifest contains 140 unique `(name, leg, debug)` keys: 32 new source families × 4 combinations plus 3 exact-reused historical jar families × 4. The four matrix combinations are Corretto 8 debug/no-debug and OpenJDK 23 debug/no-debug.

| Flavor | Full-class compile | Probe ran / passed | Reflection API matches original | API differs |
|---|---:|---:|---:|---:|
| Original | 140/140 | 140/140 | — | — |
| JADX | 130/140 | 130/140 | 126 | 4 |
| Jarde baseline | 72/140 | 72/140 | 20 | 52 |

All 140 original sources compiled and their Probe runs passed (1,040 checks, zero failures). JADX compiled and passed Probe for 130 inputs (938 checks, zero failures); ten JADX compile failures were all four BoundOverload cases, all four UnknownIncoming cases, and two MethodHandleUse cases. Jarde baseline compiled and passed Probe for 72 inputs (272 checks, zero failures). No Probe ran when that flavor failed javac. Jarde emitted 144 physical top-level class outputs (BridgeUnknown includes its helper) and all 140 requested main-class outputs had a nonempty declaration-head assertion. All 144 class-source CLI exits were zero; this does not count as compile success.

A reflection API match compares the Probe’s structured class formals/bounds, fields, constructor signatures and formals, and method formals/bounds/return/parameters. Type variables are keyed by their actual `GenericDeclaration`, formal index, and all bounds; method identities include physical return type and constructor identities include erased parameter types. The baseline mismatches are preserved in each original/JADX/Jarde Probe stdout and `reflection_sha256`; successful compile alone is not called API restoration.

| Family | Original compile | JADX compile | Jarde compile | Jarde API vs original | Behavior markers in Jarde Probe |
|---|---:|---:|---:|---:|---|
| ArrayDimensionRelay | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| ArrayRelay | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| BoundOverload | 4/4 | 0/4 | 0/4 | not probed (javac failed) | — |
| BridgeUnknown | 4/4 | 4/4 | 4/4 | 4 match / 0 differ | behavior=bridge-marker |
| CallHold | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| CallRelay | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| CatchCallMarker | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=normal-marker+caught-throw |
| CompatibleIntersectionBinder | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=Number17+identity+null-control |
| CycleRelay | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=finite-cycle-marker |
| DeepRelay | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| EmptySink | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=void-call |
| ExceptionHold | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| FieldSetter | 4/4 | 4/4 | 4/4 | 4 match / 0 differ | behavior=field-marker |
| IncompleteSite | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| IndependentCallee | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=marker |
| IndependentLeaf | 4/4 | 4/4 | 4/4 | 4 match / 0 differ | behavior=leaf-marker |
| InheritedUnknown | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=inherited-marker |
| MethodHandleUse | 4/4 | 2/4 | 4/4 | 0 match / 4 differ | behavior=method-handle-marker |
| MethodShadow | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=Number17+identity |
| MultiParam | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| MultiUseResult | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| NullCall | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| NumberBoundRelay | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| PlainUpperBoundOverload | 4/4 | 4/4 | 4/4 | 4 match / 0 differ | behavior=target-number |
| RawOwnReceiver | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| ReboundOwnReceiver | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=raw-receiver-marker |
| ReverseDeclarationRelay | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| SameErasureBinder | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=null-call-count=1 |
| SameNameOverload | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=target-generic |
| TwoClassVariables | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |
| TypedReceiverRelay | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=typed-receiver-marker |
| UnknownIncoming | 4/4 | 0/4 | 0/4 | not probed (javac failed) | — |
| VarargsCall | 4/4 | 4/4 | 4/4 | 0 match / 4 differ | behavior=varargs-marker |
| VoidDirect | 4/4 | 4/4 | 4/4 | 4 match / 0 differ | behavior=marker+call-count |
| WideRelay | 4/4 | 4/4 | 0/4 | not probed (javac failed) | — |

Target javac failures are genuine output failures in this replay. Common Jarde failure families include direct `Object -> T` call/return relays (CallRelay, arrays, multi-parameter, wide slots, deep/reverse, raw own receiver, unknown incoming), Number-bound relay (`Number -> T`), and reused CallHold/ExceptionHold. BoundOverload fails due the retained `pick(Number)` / `pick(Comparable)` ambiguity. Failures retain generated sources and javac stdout/stderr; no method was deleted to force success.

JADX compile failures are separate: UnknownIncoming has `Object -> T` at `identity(obj)`, BoundOverload retains an overload ambiguity, and MethodHandleUse has an invalid `Object -> T` method reference in two modes. These are classifier facts from exact stderr logs, not inferred from CLI status.

Probe SHA-256: `15311367877b47f1c09c950aaaa572a0ec64ad5bc497c987d842f19a3b4b807e`. Accepted Jarde CLI SHA-256: `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`. Frozen manifest SHA-256: `c01c7f3a42e1c5e759ac45e18d13d16e233df54dfc29ae2e32c27200a826496d`. Results file count: 3656 before this report.
