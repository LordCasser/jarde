# Candidate acceptance summary

Candidate CLI: `/private/tmp/jarde-generic-calls-candidate-v2-cli`  
SHA-256: `43e41e391250e53982e7d2f948ae016b486b5d62da2b6e76707576455ff77f68`  
Input rows: 140 frozen matrix (140 unique keys) + 4 nested supplement = 144; physical class-source outputs: 148.

| Gate | Status | Families |
| --- | --- | --- |
| GC-01 | needs-review | EmptySink, VoidDirect, FieldSetter, CallRelay, NullCall |
| GC-02 | needs-review | ArrayRelay, NumberBoundRelay, MultiParam, WideRelay, TwoClassVariables, ArrayDimensionRelay |
| GC-03 | needs-review | DeepRelay, ReverseDeclarationRelay, UnknownIncoming, IndependentLeaf, NestedCallArgument |
| GC-04 | needs-review | MethodShadow, IndependentCallee, CompatibleIntersectionBinder, SameErasureBinder |
| GC-05 | needs-review | TypedReceiverRelay, RawOwnReceiver, ReboundOwnReceiver |
| GC-06 | needs-review | CallHold, ExceptionHold, CatchCallMarker |
| GC-07 | pass | BoundOverload, SameNameOverload, PlainUpperBoundOverload |
| GC-08 | needs-review | UnknownIncoming, CycleRelay, MethodHandleUse, IncompleteSite, MultiUseResult, VarargsCall, BridgeUnknown, InheritedUnknown |
| GC-09 | pending-separate-regressions | field-23, constructor-80, raw-receiver-64 |
| GC-10 | pending-nonmatrix-gates | collector/type-proof/staging/output-budget/cancellation, fmt/clippy/seeds/ignored/strict-spec/real-JDK25-CI |

| Family | Group | Role | Headers | Compiles | Probe | Behavior | Full API match | Recovery pass | Refusal-control pass | Required feature | Decision |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| ArrayDimensionRelay | GC-02 | positive | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| ArrayRelay | GC-02 | positive | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| BoundOverload | GC-07 | frozen-positive-overload | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| BridgeUnknown | GC-08 | bridge-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | pass | pass |
| CallHold | GC-06 | frozen-positive | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 4/4 | 0/4 | pass | pass |
| CallRelay | GC-01 | positive | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| CatchCallMarker | GC-06 | positive-exception-path | 4/4 | 2/4 | 2/4 | 2/4 | 0/4 | 2/4 | 0/4 | fail | needs-review |
| CompatibleIntersectionBinder | GC-04 | positive-bounded-substitution | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| CycleRelay | GC-08 | finite-cycle-refusal | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| DeepRelay | GC-03 | positive-multilayer | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| EmptySink | GC-01 | positive-empty-body | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| ExceptionHold | GC-06 | frozen-positive-exception | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 4/4 | 0/4 | pass | pass |
| FieldSetter | GC-01 | carried-positive-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| IncompleteSite | GC-08 | positive-conditional-single-call | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| IndependentCallee | GC-04 | positive-independent-callee | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| IndependentLeaf | GC-03 | independent-positive-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | pass | pass |
| InheritedUnknown | GC-08 | inherited-owner-boundary | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| MethodHandleUse | GC-08 | bootstrap-boundary | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| MethodShadow | GC-04 | positive-shadowed-binder | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| MultiParam | GC-02 | positive-multiparameter | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| MultiUseResult | GC-08 | all-consumers-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 4/4 | pass | pass |
| NestedCallArgument | GC-03 | positive-nested-call-result | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| NullCall | GC-01 | positive-null-result | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| NumberBoundRelay | GC-02 | positive-bounded | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| PlainUpperBoundOverload | GC-07 | positive-overload-target | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| RawOwnReceiver | GC-05 | raw-receiver-refusal-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| ReboundOwnReceiver | GC-05 | rebound-receiver-refusal-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| ReverseDeclarationRelay | GC-03 | positive-reverse-order | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| SameErasureBinder | GC-04 | distinct-binder-boundary | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 4/4 | pass | pass |
| SameNameOverload | GC-07 | positive-overload-target | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| TwoClassVariables | GC-02 | positive-distinct-formals | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| TypedReceiverRelay | GC-05 | positive-typed-receiver | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |
| UnknownIncoming | GC-03 | mixed-incoming-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| VarargsCall | GC-08 | varargs-boundary-control | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | 0/4 | 0/4 | needs-root-review | needs-root-review |
| VoidDirect | GC-01 | carried-positive-control | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | 0/4 | pass | pass |
| WideRelay | GC-02 | positive-wide-slots | 4/4 | 2/4 | 2/4 | 2/4 | 2/4 | 2/4 | 0/4 | fail | needs-review |

## Class-source and compiler failure diagnostics

No candidate class-source errors or missing declaration heads were recorded.
- `ArrayDimensionRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/ArrayDimensionRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `e29ffd0f9b876ef9e96a7867ead53027f9a42480e94dda00887984e38e126fcf`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/ArrayDimensionRelay/compile-sources/candidate/ArrayDimensionRelay.java:26: 错误: 找不到符号
          return this.identity(x);
                               ^
    符号:   变量 x
    位置: 类 ArrayDimensionRelay<T>
- `ArrayDimensionRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/ArrayDimensionRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `ef65197d1c79b74b46925408dfc321360dc3a40e47ff020255becfa9baf81ea3`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/ArrayDimensionRelay/compile-sources/candidate/ArrayDimensionRelay.java:26: 错误: 找不到符号
          return this.identity(x);
- `ArrayRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/ArrayRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `a73f893d3388d5adea28f9f0c4dda09f9b52e90eecf802a567bea7cc3814e0ce`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/ArrayRelay/compile-sources/candidate/ArrayRelay.java:26: 错误: 找不到符号
          return this.identity(x);
                               ^
    符号:   变量 x
    位置: 类 ArrayRelay<T>
- `ArrayRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/ArrayRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `300fb5435f6904f49af4215e60357eac38cf68e0abadacdbd94f4cb4aaa72f9e`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/ArrayRelay/compile-sources/candidate/ArrayRelay.java:26: 错误: 找不到符号
          return this.identity(x);
- `CallRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/CallRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `27a4eff3d11a68c138e5de462ae08598a94dd1fb10dc1dbd8c6138bac775e740`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/CallRelay/compile-sources/candidate/CallRelay.java:26: 错误: 找不到符号
          return this.identity(x);
                               ^
    符号:   变量 x
    位置: 类 CallRelay<T>
- `CallRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/CallRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `2ef2b5f3a886a99c27d7dae64c762d63714fc268bfd7852f51793a54c415d18f`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/CallRelay/compile-sources/candidate/CallRelay.java:26: 错误: 找不到符号
          return this.identity(x);
- `CatchCallMarker` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/CatchCallMarker/compile-sources/candidate/candidate-javac.stderr` sha256 `e0b7c5aac6d946bf29afe4f5ab930b91901ce31cc201ac7b46775cf63603a54e`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/CatchCallMarker/compile-sources/candidate/CatchCallMarker.java:27: 错误: 找不到符号
          if (fail) {
              ^
    符号:   变量 fail
    位置: 类 CatchCallMarker<T>
- `CatchCallMarker` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/CatchCallMarker/compile-sources/candidate/candidate-javac.stderr` sha256 `efd41e1adda793e7dedb42aba140bbc236d8ed3ed83edea0cc354283c46da97e`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/CatchCallMarker/compile-sources/candidate/CatchCallMarker.java:27: 错误: 找不到符号
          if (fail) {
- `CompatibleIntersectionBinder` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/CompatibleIntersectionBinder/compile-sources/candidate/candidate-javac.stderr` sha256 `200692634893b1c09599436c7a2779fc0fe591bf3f9c288dd4a5c81c17837c59`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/CompatibleIntersectionBinder/compile-sources/candidate/CompatibleIntersectionBinder.java:26: 错误: 找不到符号
          return this.identity(x);
                               ^
    符号:   变量 x
    位置: 类 CompatibleIntersectionBinder<T>
- `CompatibleIntersectionBinder` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/CompatibleIntersectionBinder/compile-sources/candidate/candidate-javac.stderr` sha256 `cacfa806f57e463d80d4cec4cf208ca63928b8aad039a995923396029de3f195`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/CompatibleIntersectionBinder/compile-sources/candidate/CompatibleIntersectionBinder.java:26: 错误: 找不到符号
          return this.identity(x);
- `DeepRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/DeepRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `3de131c376186d3a7ba0352e683b14de512bb68abd1e4ba00167f42fa1806ae1`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/DeepRelay/compile-sources/candidate/DeepRelay.java:18: 错误: 找不到符号
          return this.relay1(x);
                             ^
    符号:   变量 x
    位置: 类 DeepRelay<T>
- `DeepRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/DeepRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `3016b84666d81e1d18460518620e4c06dd237aa699be21a7f454a8dcdc747eb3`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/DeepRelay/compile-sources/candidate/DeepRelay.java:18: 错误: 找不到符号
          return this.relay1(x);
- `IncompleteSite` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/IncompleteSite/compile-sources/candidate/candidate-javac.stderr` sha256 `b033dcba1461fa86cde1bd4286391e7abf380724f191c29309f011ec539962f7`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/IncompleteSite/compile-sources/candidate/IncompleteSite.java:26: 错误: 找不到符号
          if (use) {
              ^
    符号:   变量 use
    位置: 类 IncompleteSite<T>
- `IncompleteSite` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/IncompleteSite/compile-sources/candidate/candidate-javac.stderr` sha256 `8516cda15cf07bf697912f598abc338daf45554c76335b1762d4743b58ebd056`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/IncompleteSite/compile-sources/candidate/IncompleteSite.java:26: 错误: 找不到符号
          if (use) {
- `IndependentCallee` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/IndependentCallee/compile-sources/candidate/candidate-javac.stderr` sha256 `411579ca6fab3886e08236c0cae74a0156a9084fb58612fbb8ad816ab20fd826`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/IndependentCallee/compile-sources/candidate/IndependentCallee.java:26: 错误: 找不到符号
          return this.identity(x);
                               ^
    符号:   变量 x
    位置: 类 IndependentCallee<T>
- `IndependentCallee` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/IndependentCallee/compile-sources/candidate/candidate-javac.stderr` sha256 `cb94e81a513fe6c368fa7c5f879a383a670448305d66335640d4b0b1733103cd`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/IndependentCallee/compile-sources/candidate/IndependentCallee.java:26: 错误: 找不到符号
          return this.identity(x);
- `MultiParam` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/MultiParam/compile-sources/candidate/candidate-javac.stderr` sha256 `4ac5e29a0dba30fb432c6fca6ca120a04696d3b7b122c326ebea13709a3900b0`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/MultiParam/compile-sources/candidate/MultiParam.java:26: 错误: 找不到符号
          return this.first(x, y);
                            ^
    符号:   变量 x
    位置: 类 MultiParam<T>
- `MultiParam` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/MultiParam/compile-sources/candidate/candidate-javac.stderr` sha256 `9b3ba62a274b51c21fbf2c369d8b22edc648b59d099d3d196b5f92d04c1856ee`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/MultiParam/compile-sources/candidate/MultiParam.java:26: 错误: 找不到符号
          return this.first(x, y);
- `NestedCallArgument` corretto8 debug: `` sha256 ``
- `NestedCallArgument` openjdk23 debug: `` sha256 ``
- `NumberBoundRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/NumberBoundRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `1feb128f7ab9c264c7bb31c8caf6b3bfa59ff773ca40954f1015428fafe74104`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/NumberBoundRelay/compile-sources/candidate/NumberBoundRelay.java:26: 错误: 找不到符号
          return this.identity(x);
                               ^
    符号:   变量 x
    位置: 类 NumberBoundRelay<T>
- `NumberBoundRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/NumberBoundRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `008852bdbf8b258d07f87657b2e12a6b80294edfab2682f00cfa2666d82c4d2e`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/NumberBoundRelay/compile-sources/candidate/NumberBoundRelay.java:26: 错误: 找不到符号
          return this.identity(x);
- `ReverseDeclarationRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/ReverseDeclarationRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `02b61f81b620ee8697f9441b81b9e2fe968891803044d201c258f4cefb1368e9`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/ReverseDeclarationRelay/compile-sources/candidate/ReverseDeclarationRelay.java:26: 错误: 找不到符号
          return this.identity(x);
                               ^
    符号:   变量 x
    位置: 类 ReverseDeclarationRelay<T>
- `ReverseDeclarationRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/ReverseDeclarationRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `3d363bd7ee24811496f30fa377a93d439fe8ddcfcd0daf7f1bd214be7c62fd3a`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/ReverseDeclarationRelay/compile-sources/candidate/ReverseDeclarationRelay.java:26: 错误: 找不到符号
          return this.identity(x);
- `TwoClassVariables` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/TwoClassVariables/compile-sources/candidate/candidate-javac.stderr` sha256 `2fbe5cf657d3e7d2830c4a453c3f0b4af296f0123d9d6c6293be13fa11c10f3a`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/TwoClassVariables/compile-sources/candidate/TwoClassVariables.java:26: 错误: 找不到符号
          return this.combine(a, b);
                              ^
    符号:   变量 a
    位置: 类 TwoClassVariables<A,B>
- `TwoClassVariables` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/TwoClassVariables/compile-sources/candidate/candidate-javac.stderr` sha256 `6b315de5f044f88fbb9ee71ec0bd0585381fe06d218e6d3a3a2fa676e6e26a04`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/TwoClassVariables/compile-sources/candidate/TwoClassVariables.java:26: 错误: 找不到符号
          return this.combine(a, b);
- `TypedReceiverRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/TypedReceiverRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `ae62be5489563a20d8aa451fabd9504eb62b6bab68430a999c48b62d2ce8b8cf`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/TypedReceiverRelay/compile-sources/candidate/TypedReceiverRelay.java:26: 错误: 找不到符号
          return receiver.identity(x);
                                   ^
    符号:   变量 x
    位置: 类 TypedReceiverRelay<T>
- `TypedReceiverRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/TypedReceiverRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `38b618c24c4ab45e538ddc19b1e6a0202c11090b29b0a14067a1b116fd44440b`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/TypedReceiverRelay/compile-sources/candidate/TypedReceiverRelay.java:26: 错误: 找不到符号
          return receiver.identity(x);
- `WideRelay` corretto8 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/WideRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `0ad58f3a6954de2037612ba14922f89f2b665aa4ff64309516eefee6f5fd8490`
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/corretto8/debug/WideRelay/compile-sources/candidate/WideRelay.java:26: 错误: 找不到符号
          return this.identity(y);
                               ^
    符号:   变量 y
    位置: 类 WideRelay<T>
- `WideRelay` openjdk23 debug: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/WideRelay/compile-sources/candidate/candidate-javac.stderr` sha256 `f347e48080addda30dba2afa5b0c70a2e75f5f9740b411deea16110f8319404a`
  警告: [options] 源值 8 已过时，将在未来发行版中删除
  警告: [options] 目标值 8 已过时，将在未来发行版中删除
  警告: [options] 要隐藏有关已过时选项的警告, 请使用 -Xlint:-options。
  /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v2/gc01-08-140/openjdk23/debug/WideRelay/compile-sources/candidate/WideRelay.java:26: 错误: 找不到符号
          return this.identity(y);

GC-09 and GC-10 remain pending their separate required runs and root checks. This report treats CLI status, full-class compilation, Probe behavior, generic reflection, and failure stderr as separate evidence.
