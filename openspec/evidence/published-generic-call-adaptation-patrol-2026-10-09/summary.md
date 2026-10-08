# 已发布泛型 callee 的实参适配：root 独立巡查

2026-10-09，基线 main `564e22c1`，使用构造器片最终已验收 CLI `/tmp/jarde-class-ctor-final-cli`，SHA256 `3e7241e99b0068ede015b4dbc419212f5a4f63cb9a78f37429662b34f6d7a48c`。本次不改生产、不运行 Cargo、无新 worktree。输入是前片冻结的真实 Corretto8/OpenJDK23、debug/no-debug 的 CallHold/ExceptionHold 原 jar，不重新生成不同字节码。

八输入的新独立完整重编：原源码 8/8、已冻结的 JADX 完整源码 8/8 均成功，在实际 VM `-Xverify:all` 下字段值与传入 marker 是同一对象；当前 main CLI 输出 8/8 非空、有声明头、返回0，但完整 Java 8 重编 0/8，失败全文保留。所有编译显式空 classpath/sourcepath，运行仅新的 classes，不含原 jar 或原 class。JADX 使用前片真实 CLI 保存的源码，本次重新编译执行，不宣称另跑了 JADX CLI。

原例：`CallHold<T>(T v) { this.v=identity(v); } private T identity(T x) { return x; }`。Jarde 正文正确表达 `this.v=this.identity(v)`，callee 的头已发布 `T identity(T x)`，constructor 头仍是 Object，javac 报 Object 无法转换为 T。ExceptionHold 同样，区别是 constructor 有 EH，正文也已结构化恢复。字段 T 被拒是调用产生值的 RHS 不属于当前 closed source；**整类编译失败的直接原因是调用实参类型，不能称为正文未恢复或字段声明问题**。

源码架构审查：`src/class_source.rs::prove_same_class_method_binding` 通过 own class/Object父类/无interface、同名不同arity、完整invoke/memberref清单证明 source binding，但相同descriptor的invoke直接通过，未证明实际发射参数能赋给新发布的 callee 参数。该身份门不等于泛型实参适配门。`ordinary_parameterized_declaration` 的 same-run 参数-return候选允许 identity T→T，而 constructor 当前直接字段初始化候选排除调用；两个独立发布局部正确，组合仍可能不可编译。

下一片方向是保留按物理调用位/AST实参/参数完整使用的事实，在已证明并实际发布的同类 callee 类型下闭合 caller 的泛型参数与调用结果。不以被拒绝 Signature 或同名变量猜来源，不默认插 cast，也不通过一律擦除已证明 callee 来掩盖 API 缺口。先审阅 JADX 的 TypeUpdate invoke参数替换与 TypeUtils.getTypeVarMappingForInvoke，构造普通relay/void调用、raw receiver、bound/array、多形参、方法binder、不同callee、异常边界，区分调用身份与类型适配后立 OpenSpec。是否需要候选依赖闭包应从这些实际反例决定；本片证据尚不能批准通用 fixpoint 或整个 generic 单元完成。

该债务与当前 `recover-raw-receiver-field-selection` 独立；不要混入 raw 字段实现。可重放：

```sh
python3 openspec/evidence/published-generic-call-adaptation-patrol-2026-10-09/replay.py --cli /tmp/jarde-class-ctor-final-cli --out /tmp/new-call-adaptation-replay
```

`results/manifest.json` 保存实际参数列表、返回码、输入/源哈希及逐例结果；源文件和原jar引用前片冻结路径，结果目录保存完整baseline源及编译/运行输出，生成classes均已清理。

## 普通 void body 与重载边界补核

root按实际源码复核：`void set(T x) { this.value=x; }`已有 `GenericReturnValue::VoidBody` 路径，候选不要求空body，但 `ordinary_parameterized_declaration` 只接受所有参数均为直接class作用域TypeVariable、无throws的完整直线void body。不能泛化到 `C<T>` receiver、混合boolean参数或EH，也不应重复发明setter候选。getter返回字段是另一方向的依赖，当前方法先发布、字段后发布，需另片分析。

`VoidBody` 的“参数未重写”并不证明其所有调用消费位：新增 [BoundOverload四腿](overload-control/strict-repo-jadx-results/summary.md) 已严格复现 `T extends Number & Comparable<T>` 的 `relay(T x){pick((Number)x);}`。原字节码四腿皆为 `pick(Number)`，无checkcast；原完整源码4/4打印number。Jarde发布relay(T)但仍发射pick(x)，两个重载歧义，0/4完整编译。参考仓库构建的JADX(dev，57个lib jar hash已记录)与Homebrew JADX1.5.6独立各0/4完整编译；Jarde的Comparable参数仍是raw，JADX保留Comparable<T>，失败全文与完整类都保留。root独立核对两份四腿manifest共214个结果文件及两版脚本hash。根overload-control的早期复用-d运行仅作初步诊断，不计正式结果。

JADX `TypeUpdate.invokeListener` 与 `InvokeUpdateCallback` 分别替换receiver类变量、invoke参数/结果；`TypeUtils.getTypeVarMappingForInvoke` 只处理直接type-var映射并明示嵌套List<T>映射TODO。`MethodInvokeVisitor.processOverloaded` 根据compiler arg types与已选target尝试pin/cast，可借其消费位思路，但上述真实失败说明还须连接泛型头恢复。参考具体测试 `TestGenericsInArgs::test/testNoDebug`、`TestGenericFields::test`、`TestGenerics7::test`。它们不是Jarde已通过的验收。下一片先冻结最小调用实参/返回、binder与重载控制，确定已有AST侧车可复用范围，再决定是否需要新事实；这些证据尚不要求通用fixpoint。

## 下一片的有限单腿分类

[mini-patrol](mini-patrol/summary.md) 是CI等待期间的Corretto8 debug exploratory，不是四腿最终验收。原/JADX四类全部完整编译、反射和marker均符合预期。Jarde的FieldSetter已有set(T)/fieldT完整闭环；EmptySink未用T参数仍被擦除；TypedSetter的C<T>/T方法参数被擦除，但raw字段T恢复、行为保持；CallRelay的identity(T)已发布，relay仍按Object发射，完整类因实参Object不能传给T而失败。root逐项核对102个结果文件hash后保留源码/jar、实际CLI输出和日志；历史命令路径仍指向实际运行的/tmp位置，脚本可复制到新的空/tmp目录重放，不覆盖已保存目录。

对参考JADX的追加静态核对发现两个条件性失口：若invoke阶段compiler type仍是擦除Number，重载唯一匹配早退且同型不补cast；若类型已是intersection T而bound与目标Comparable表示不精确相等，TypeCompare在未命中extendTypes.contains时要求所有bounds都narrow，遇首个非narrow提前返回，也可能错误排除另一个适用目标。四腿输出不揭示实际IR状态，未做动态IR调试，不能宣称本例确定命中哪条。Jarde下一片以自己的实际published caller/callee类型、实参来源、物理invoke目标闭合证明，不直接照搬该comparator。
