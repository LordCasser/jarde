# CLI-v6 控制事实摘录（中文）

本文只整理固定 CLI-v6 的物理与源码证据，供 root 逐项复核；不替 root 作最终裁定。完整逐腿数据在 [`candidate-v6-control-facts.json`](candidate-v6-control-facts.json)，SHA-256：`42e3a88e56c2a1f4c242b9f3d65e41c6a5cf704295b44fafe1730e883c31dd1b`。每个被审查族均有 corretto8/debug、corretto8/nodebug、openjdk23/debug、openjdk23/nodebug 四条记录；行号指向各自实际保存的 candidate-v6 输出。七个 `CONTROL_REVIEW_ONLY` 项仍标为待 root 判断。编译/Probe/行为相符、marker 或反射 API 差异均不单独证明拒绝正确。

固定输入为 CLI `/private/tmp/jarde-generic-calls-candidate-v6-cli`（SHA-256 `c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8`），源码快照 v38 archive SHA-256 `83b1da417d94bcbcfd83d8871a053be93aa31cb2d2e6e3e05f215699db5a5f11`。重放使用固定外层 runner [`candidate-acceptance-replay-executed.py`](../candidate/candidate-v6/candidate-acceptance-replay-executed.py)（SHA-256 `a19895e70ef306a3024132a3da7c07cf891ea87e5ef0e25e01413989514ce836`）；run manifest SHA-256 `028f94f3d9e2505322f710b9e58c234e329682b857d718627b13dda9a56dd92d`，列出的文件共 3836 个，完整性错误 0 个。候选矩阵 140 项、Nested 补充 4 项，共 144 行，148 个物理类源码输出；144/144 编译、Probe、行为比对通过，完整反射匹配 96/144，矩阵结论仍是 `needs-review`。这些总数不意味着所有控制拒绝都正确。

逐腿的输入源码、input jar、candidate 源码及哈希、Probe 原始输出、输出源文件行和固定回放 manifest record 都在 JSON。输入 jar 与上一轮 CLI-v5 所用 frozen jar 逐条哈希相同；因此 `input_javap.excerpt_lines` 沿用对同一物理 jar 的只读 `javap -v -p` 记录，不是从族名推断。下文同时给出源码、AST 文本和 classfile 调用事实的互相对应。所有拒绝边界均限于证据能支持的部分。

## 七项待 root 复核的 CONTROL_REVIEW_ONLY
**UnknownIncoming（GC-03/08 混合 incoming-use）**：原源码只有 `identity(T)`、`safe(T)`、`untouched(T)` 和 `unsafe(Object)`；`safe` 调 `identity(x)`，`unsafe` 调 `identity((T)x)`，而 `untouched` 是独立的 `T→T`。源码原文在每腿 `input-source/UnknownIncoming.java:1`。四腿候选均保留 class `UnknownIncoming<T>`，但 `identity(Object):Object` 前有“same-class generic call dependency did not close over every incoming use”拒绝标记；safe/unsafe 的 Object 头有 `ordinary_generic_source_unproved` 标记，AST 对应为 `return this.identity(...)`（debug 输出 `UnknownIncoming.java:13-14,21-26,37-42`；nodebug 行号/参数名见各自文件）。冻结 classfile 的 `javap` 在物理偏移证据行 85/125 显示两处 `invokevirtual identity:(Object)Object`。原源码与调用目标一致；candidate 将两条 consumer 都呈现成 Object 返回/参数，`untouched(T)` 仍单独保留。每腿原始与 candidate Probe 均通过且行为行相符，完整反射 API 不匹配（0/4 full reflection）。这支持检查所有 incoming use 都已计入的边界；它本身未证明拒绝范围是否恰好正确。

**CycleRelay（GC-08 有限互递归边界）**：原源码 `left(T,boolean)` 在分支中调 `right(x,false)`，`right` 又调 `left(x,false)`，两个成员的静态签名及完整源码在每腿 `input-source/CycleRelay.java:1`。候选四腿均将 left/right 头呈现为 `Object` 并分别标 `ordinary_generic_source_unproved`；AST 保留 `this.right(x,false)` 与 `this.left(x,false)`（debug `jarde/CycleRelay.java:13-31`）。物理 `javap` 对应为 `invokevirtual right:(Object,Z)Object`（行 71）和 `invokevirtual left:(Object,Z)Object`（行 100），不是仅凭方法名推定的递归。Probe/行为四腿通过，full reflection 0/4。现有物理证据确认循环调用存在、候选拒绝两成员；该回放标为待复核，不能把通用 `ordinary_generic_source_unproved` marker 当作循环拒绝已满足判据。

**MethodHandleUse（GC-08 bootstrap / method-reference 边界）**：原源码 `relay(T)` 保存 `Function<T,T> f=this::identity` 后调用 `f.apply(x)`，见每腿 `input-source/MethodHandleUse.java:1`。candidate 的 `identity(Object)` 带 `generic_call_binding_unproved`，relay 带 `ordinary_generic_source_unproved`；AST 在 debug 输出 `Function f = this::identity; return f.apply(x);`（`jarde/MethodHandleUse.java:13-27`）。input jar 的 `javap` 证据包含 `invokedynamic ... LambdaMetafactory.metafactory`（第108、132-133行）、bootstrap handle `invokevirtual MethodHandleUse.identity:(Object)Object`（第42、136行）以及 `invokeinterface Function.apply`（第112行），把源码 method reference 和间接句柄实际连起来。四腿 Probe/行为通过而 full reflection 0/4。该证据确认还存在 bootstrap/handle consumer；是否应因此拒绝这些具体签名仍留给 root。

**RawOwnReceiver（GC-05 raw receiver 边界）**：原源码显式声明 raw 局部 `RawOwnReceiver raw=this`，随后返回 `(T)raw.identity(x)`，源码在每腿 `input-source/RawOwnReceiver.java:1`。物理 classfile 有 `invokevirtual identity:(Object)Object`（`javap` 行85）。CLI-v6 候选 `relay(Object):Object` 带 `ordinary_generic_source_unproved`，但 AST 文本是 `return this.identity(x)`（debug `jarde/RawOwnReceiver.java:21-26`；其他腿对应文件相同结构、参数名可能为 `arg1`）；它没有保留源中的 raw 局部，也没有呈现 raw 变量作为 receiver。因此候选输出不能单独证明它正确保留了源 raw receiver 语义。四腿 Probe/行为通过，full reflection 0/4。原始局部+物理调用是拒绝审查的支持材料，不能据此把候选 marker 当作充分拒绝证据。

**ReboundOwnReceiver（GC-05 重绑定边界）**：务必区分源码静态类型与 raw cast：原源码先声明 `ReboundOwnReceiver<T> receiver=this`，赋给 `Object alias`，再执行 `receiver=(ReboundOwnReceiver)alias`。这个 unchecked assignment 不会把变量的静态声明类型改成 raw；源码原文位于每腿 `input-source/ReboundOwnReceiver.java:1`。因此不能以“原源码最后一次调用必然按 raw receiver 选择”为拒绝理由。冻结 classfile 的 `javap` 行91 是 `invokevirtual identity:(Object)Object`。CLI-v6 candidate debug 输出在 `jarde/ReboundOwnReceiver.java:26-29` 有 erased/raw 呈现的 `ReboundOwnReceiver receiver = this; ReboundOwnReceiver alias = receiver; receiver = (ReboundOwnReceiver) alias; return receiver.identity(x);`；nodebug 则以 `local2/local3` 保存同类 erased/raw AST，见该腿 `jarde/ReboundOwnReceiver.java:26-30`。这描述的是候选 AST 局部变量声明，并不改变原源码 `receiver` 的静态类型事实。relay 仍有 `ordinary_generic_source_unproved` 标记；四腿 Probe/行为通过而 full reflection 0/4。可审查的支持边界是原源码中的 Object alias、unchecked raw cast/重绑定与候选 raw 局部呈现；实际拒绝正确性待 root 判断。

**InheritedUnknown（GC-08 inherited-owner 边界）**：原源码类声明 `InheritedUnknown<T> extends ArrayList<T>`，`relay(int)` 返回继承的 `get(index)`，见每腿 `input-source/InheritedUnknown.java:1`。物理 input jar 的调用为 `invokevirtual java/util/ArrayList.get:(I)Object`（`javap` 行62）。候选 class 头擦除成 `InheritedUnknown extends java.util.ArrayList`，relay 头为 `Object`，并在方法前标 `jvm_signature_scope_unproved`，明确称 `T` 不属于可用 Signature scope；AST 为 `return this.get(index)`（debug `jarde/InheritedUnknown.java:5,14-19`）。四腿 Probe/行为通过，full reflection 0/4。证据确认方法 owner 是父类以及候选 scope 标记，但还需 root 按签名/继承边界判据核拒绝范围。

**VarargsCall（GC-08 varargs 边界）**：原源码的 `first` 是 `@SafeVarargs final <U> U first(U... xs)`，relay 中显式写 `this.<T>first(x)`，源码在每腿 `input-source/VarargsCall.java:1`。物理 javap 显示 `first` 的 flags 含 `ACC_PUBLIC, ACC_FINAL, ACC_VARARGS`（第65行）；relay 调用是 `invokevirtual first:([Object])Object`（第98行）。CLI-v6 candidate 保留 `first(Object... xs)` 头，但在它前面标 `generic_call_binding_unproved`；relay 的 Object 头标 `ordinary_generic_source_unproved`，AST 为 `return this.first(x)`（debug `jarde/VarargsCall.java:13-27`），没有呈现原源码的显式 `<T>` 选择。四腿 Probe/行为通过、full reflection 0/4。此处可见 varargs 位、数组描述符、显式类型调用与候选调用呈现；是否以此边界拒绝以及范围是否恰当仍由 root 裁定。

## 需要隔离的其他控制

`SameErasureBinder` 与 `MultiUseResult` 是固定 replay 判据通过的两类拒绝控制（各 4/4）；事实 JSON 保留了各腿完整输出和拒绝判据记录。它们不能替代上面的七项人工复核。`BridgeUnknown` 是独立正 bridge control（4/4 boundary control pass），`IncompleteSite` 是合法单调用的正向条件恢复（4/4 feature recovery pass）；两者均不应混入缺失 consumer/拒绝控制。完整逐项信息见 JSON 中对应 `review_role`、`saved_replay_predicate` 与 candidate 输出。

## root 独立核验入口

matrix：

```sh
python3 openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/verify-candidate-matrix.py --results openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v6/gc01-08-140 --cli-sha256 c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8 --out openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/candidate-v6-matrix-verification-root.json
```

Nested：

```sh
python3 openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/verify-nested-candidate.py --results openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v6/nested-call-4 --cli-sha256 c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8 --out openspec/changes/recover-same-class-generic-call-consumers/results/root-input-audit/candidate-v6-nested-verification-root.json
```

这两个命令尚未由本摘录执行；verifier 结果留给 root 独立生成。
