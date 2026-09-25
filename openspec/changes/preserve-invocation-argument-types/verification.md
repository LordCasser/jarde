# 调用参数类型：实施与独立验收

2026-09-23。生产规则、实际整类执行和相邻回归已由主代理验收。严格 clippy 的既存区域债务独立记录，功能验收不表示全仓全部门禁通过。

## 冻结输入与证据口径

永久输入 `tests/fixtures/p3-invocation-arguments/v8/InvocationArgumentsProbe.class`：一个class、14个Code方法，SHA-256为 `f4c364f1ecb388c64c4ae9140df820ad625a43e3b158426698865e4fe43bedde`。以javac 23.0.1、`--release 8 -g:none`重新编译当前自写源码，与冻结class逐字节相同。辅助类和runner保留源码，尤其保留GenericFactory的`<T> T make()`泛型声明。

最终三方证据在 `../../evidence/java-syntax-2026-09-22/overloads/final-fixture/`。原class、实际jarde输出、实际jadx输出独立编译并以`java -Xverify:all`执行。jadx自带`package defpackage;`未改，只给临时helper/runner添加相同package。两种生成方法体均未替换、删除或补全。

旧`/tmp/jarde-invocation-arguments-20260923/jarde.java.txt`是初版fixture输出，数组初始化与i2l仍引用，不能完整编译。此前将最终fixture推断值写成“修前整类实际执行”缺乏证据，该表述已删除。修前真实可编译错值由`overloads/independent/before.txt`等独立证据支持，不能混用不同fixture版本。

## 最终fixture三方执行

| 方法 | 原class | 当前jarde | jadx 1.5.6 |
| --- | --- | --- | --- |
| objectString | 3 | 3 | 4 |
| objectNull | 3 | 3 | 3 |
| objectArray | 5 | 5 | 6 |
| objectBoxed | 7 | 7 | 8 |
| widening | 10 | 10 | 9 |
| narrowByte | 11 | 11 | 11 |
| narrowShort | 12 | 12 | 12 |
| constructorObject | 1 | 1 | 2 |
| multiple | 15 | 15 | 14 |
| genericObject | 3 | 3 | 4 |
| functionalRunnable | 16 | 16 | 17 |
| functionalObject | 18 | 18 | 19 |

jarde 12/12一致，jadx 9项不同。这是冻结样例的结论，不是任意重载/泛型保证。方法引用先固定已证明的Runnable工厂类型，再在Object参数处保留外层Object类型。

## 主代理独立输入

`overloads/independent/`额外整类有12行输出。修前jarde完整编译但有3个错值：super/this构造器均1→2，多参数trace仍为12但目标返回41→42。重建debug CLI后整类原样编译，12/12一致。Object/String同型局部、显式cast、null与CCE为稳定对照，外部helper保留原声明。

`overloads/independent/self-argument/`揭示隐式this缺少presented类型。复用声明类事实后，实际输出`pick((java.lang.Object) this)`，整类重编译执行1，与原class相同。没有调用专用SSA类型推断。

`overloads/type-boundaries/`将接口反例唯一checkcast等宽替换为三个nop。root重跑`-Xverify:all`，Object进入不使用参数的Runnable形参方法，返回7。当前jarde明确引用该调用，没有加入可能导致CCE的Runnable cast。此次证据为`root-current-jarde.*`、`root-patched-current.txt`。

## 架构审查

复用Type、Expr.presented和Cast，无新AST/pass/resolver/library。parameter_types由同一descriptor保留精确引用/数组类型及slot布局，替代历史boolean专用Object占位；this由声明类提供类型。不可读descriptor、参数数量不符、类型事实缺失和未知引用关系均拒绝；null与上溯Object为安全边界。同型稳定表达式不多包装，poly调用和函数式目标明确限定。cast移动持有的表达式，保留生产者来源并加调用BCI的derived锚点；最终消费点的求值校验与失败生产者引用协议不变。

## 回归与剩余门禁

- `/tmp/jarde-invocation-root-current-jdk.log`：真实Engine整类输出JDK测试1通过。
- `/tmp/jarde-root-invocation-adjacent-bodies.log`：调用参数3、eval-context10、receiver4、new-value4、reference-cast5、special-dispatch3、special-refusal4、static-call1通过。
- meeting2项、required-conversions1项旧无cast断言修正后，`/tmp/jarde-root-invocation-text-expectations.log`两套14项通过；返回/赋值隐式加宽不变。
- source-map测试实际请求全部证据，验证调用BCI同时有direct与derived来源。
- patterns 的缺声明类型输入已改为读取真实 DeclaringClass；两个匿名 use-site 的 owner 修正为 Outer（池项仍为 Outer$1），max_stack 修为真实所需 3。刻意缺声明类的测试仍保留独立入口。未放松生产引用边界。
- class-source旧ExplanationOnly样例因普通cast恢复失效，换为合法未支持dup形状后，`/tmp/jarde-root-class-source-refresh.log`16项通过。
- census为`(88,480,74,227,8)`，fingerprint205项（8新增、0修改、0删除），详见审计corpus目录。
- 最终 `/tmp/jarde-root-java-final-invocation-static.log`：176项通过（98单元、32 recovery、46 patterns）。新增的未知接口拒绝和this上溯Object测试真实校验非空来源、消费BCI与物理成员。
- library class-source16、CLI16、declarations5、D1 evidence selection8、带test-support的D0 instrumented5均通过；来源默认关闭、证据选择独立及实际来源预算停止已有回归。日志已归档到 `overloads/root-regressions/`。
- `/tmp/jarde-root-clippy-next-final.log`：严格clippy在既存`region.rs:1736 type_complexity`失败，未加allow或混入区域重构；不能宣称all-targets已全部检查通过。
- 最终全仓fmt通过，OpenSpec strict34项通过；日志与patterns46项通过记录归档在 `overloads/root-regressions/`。本项3.2按“执行门禁并区分既存债务”的验收要求完成；区域lint债务仍在审计总表，未消失也未被本项修复。
