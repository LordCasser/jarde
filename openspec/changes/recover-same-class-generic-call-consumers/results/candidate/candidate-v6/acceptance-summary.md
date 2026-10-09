# Candidate acceptance summary

Candidate CLI: `/private/tmp/jarde-generic-calls-candidate-v6-cli`  
SHA-256: `c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8`  
Input rows: 140 frozen matrix (140 unique keys) + 4 nested supplement = 144; physical class-source outputs: 148.

| Gate | Status | Families |
| --- | --- | --- |
| GC-01 | pass | EmptySink, VoidDirect, FieldSetter, CallRelay, NullCall |
| GC-02 | pass | ArrayRelay, NumberBoundRelay, MultiParam, WideRelay, TwoClassVariables, ArrayDimensionRelay |
| GC-03 | needs-review | DeepRelay, ReverseDeclarationRelay, UnknownIncoming, IndependentLeaf, NestedCallArgument |
| GC-04 | pass | MethodShadow, IndependentCallee, CompatibleIntersectionBinder, SameErasureBinder |
| GC-05 | needs-review | TypedReceiverRelay, RawOwnReceiver, ReboundOwnReceiver |
| GC-06 | pass | CallHold, ExceptionHold, CatchCallMarker |
| GC-07 | pass | BoundOverload, SameNameOverload, PlainUpperBoundOverload |
| GC-08 | needs-review | UnknownIncoming, CycleRelay, MethodHandleUse, IncompleteSite, MultiUseResult, VarargsCall, BridgeUnknown, InheritedUnknown |
| GC-09 | pending-separate-regressions | field-23, constructor-80, raw-receiver-64 |
| GC-10 | pending-nonmatrix-gates | collector/type-proof/staging/output-budget/cancellation, fmt/clippy/seeds/ignored/strict-spec/real-JDK25-CI |

| Family | Group | Role | Headers | Compiles | Probe | Behavior | Full API match | Recovery pass | Refusal-control pass | Required feature | Decision |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| ArrayDimensionRelay | GC-02 | positive | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| ArrayRelay | GC-02 | positive | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| BoundOverload | GC-07 | frozen-positive-overload | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| BridgeUnknown | GC-08 | bridge-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | pass | pass |
| CallHold | GC-06 | frozen-positive | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 4/4 | 0/4 | pass | pass |
| CallRelay | GC-01 | positive | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| CatchCallMarker | GC-06 | positive-exception-path | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 4/4 | 0/4 | pass | pass |
| CompatibleIntersectionBinder | GC-04 | positive-bounded-substitution | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| CycleRelay | GC-08 | finite-cycle-refusal | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| DeepRelay | GC-03 | positive-multilayer | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| EmptySink | GC-01 | positive-empty-body | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| ExceptionHold | GC-06 | frozen-positive-exception | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 4/4 | 0/4 | pass | pass |
| FieldSetter | GC-01 | carried-positive-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| IncompleteSite | GC-08 | positive-conditional-single-call | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| IndependentCallee | GC-04 | positive-independent-callee | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| IndependentLeaf | GC-03 | independent-positive-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | pass | pass |
| InheritedUnknown | GC-08 | inherited-owner-boundary | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| MethodHandleUse | GC-08 | bootstrap-boundary | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| MethodShadow | GC-04 | positive-shadowed-binder | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| MultiParam | GC-02 | positive-multiparameter | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| MultiUseResult | GC-08 | all-consumers-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 4/4 | pass | pass |
| NestedCallArgument | GC-03 | positive-nested-call-result | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| NullCall | GC-01 | positive-null-result | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| NumberBoundRelay | GC-02 | positive-bounded | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| PlainUpperBoundOverload | GC-07 | positive-overload-target | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| RawOwnReceiver | GC-05 | raw-receiver-refusal-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| ReboundOwnReceiver | GC-05 | rebound-receiver-refusal-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| ReverseDeclarationRelay | GC-03 | positive-reverse-order | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| SameErasureBinder | GC-04 | distinct-binder-boundary | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 4/4 | pass | pass |
| SameNameOverload | GC-07 | positive-overload-target | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| TwoClassVariables | GC-02 | positive-distinct-formals | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| TypedReceiverRelay | GC-05 | positive-typed-receiver | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| UnknownIncoming | GC-03 | mixed-incoming-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| VarargsCall | GC-08 | varargs-boundary-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| VoidDirect | GC-01 | carried-positive-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| WideRelay | GC-02 | positive-wide-slots | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |

## Class-source and compiler failure diagnostics

No candidate class-source errors or missing declaration heads were recorded.
No candidate whole-class compile failures were recorded.

GC-09 and GC-10 remain pending their separate required runs and root checks. This report treats CLI status, full-class compilation, Probe behavior, generic reflection, and failure stderr as separate evidence.
