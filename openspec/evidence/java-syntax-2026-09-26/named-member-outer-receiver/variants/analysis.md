# `Outer` 值流与嵌套语法边界

本探针用 `javac --release 8 -g` 编译。主正例的外层 `state` 为 10，显式参数 `other.state` 为 20；`OuterReceiverCases.super.value()` 返回 1，成员对象的 `this.value()` 从 `ReceiverMemberBase` 返回 3。原 class 运行结果为 `20:10:1:3`，见 [original-run.txt](original-run.txt)。同一行同时区分了两个同类型 `OuterReceiverCases` 实例、限定外层 `super` 和当前 `this` 的分派目标。

`javap-Member.txt` 显示 `compare(other)` 对第一个值把局部参数直接传给 `OuterReceiverCases.access$000`，对第二、三个值则先从成员对象读取 `this$0` 再传给访问器。`javap-Outer.txt` 显示 `access$000` 读接收对象的 `state`，而 `access$101` 的唯一调用是 `invokespecial ReceiverBase.value`。所以同一个 `OuterReceiverCases` 静态类型对应了不同对象值；仅凭类型，或字段名 `this$0`，不足以把 `other` 当作词法外层对象。当前 Jarde 的 [Member 输出](jarde-Member.java)仍保留两条值流：`access$000(other)` 与 `access$000(this.this$0)`；它还保留 `access$101(this.this$0)` 和当前对象的 `this.value()`。这份单类输出明确不承诺可编译，且没有输出 `OuterReceiverCases.this` / `OuterReceiverCases.super`。

同 jar 还包含合法的 static nested class 和一个 local class。合法 static nested class 仅通过显式 `other` 参数读外层字段，不携带 `this$0`；[StaticNested 输出](jarde-StaticNested.java)也没有发明词法外层访问。单独的 [非法 static 例](StaticNestedIllegal.java)在 static nested body 中尝试 `Outer.this` 与 `Outer.super`，编译器分别报告静态上下文不能引用 `this` 和 `super`，见 [static-nested-javac.txt](static-nested-javac.txt)。

local class 不是同一类语法负例：Java 允许其中使用 `OuterLocalBoundary.this` 和 `OuterLocalBoundary.super`。原 class 运行 `20:10:1`。`javap-Local.txt` 中它同时有 `val$other` 与 `this$0` 字段，并带 `EnclosingMethod: OuterLocalBoundary.compareWithLocal`；方法分别从两个字段取值后调用外层访问器，证明同型捕获对象和词法外层实例依旧要分开。当前 Jarde 的 [Local 输出](jarde-Local.java)也保留这两个不同字段，但仍以 `$` 类名和 accessor 调用呈现。

JADX 1.5.6 对命名成员正例能生成并编译整份 `OuterReceiverCases.java` 及两个基类，重编结果同为 `20:10:1:3`（[输出](jadx-generated/defpackage/OuterReceiverCases.java)、[编译日志](jadx-positive-javac.txt)及 [运行结果](jadx-positive-run.txt)）。它恢复为 `other.state`、`OuterReceiverCases.this.state`、`OuterReceiverCases.super.value()` 和 `value()`。JADX 对 local class 则输出 `new Object(this) { ... }`，并把 Outer 值赋给匿名 `Object` 的 `this$0`；全套反编译源码因此无法编译，诊断见 [jadx-javac.txt](jadx-javac.txt)。该 local 结果是本探针中 JADX 的已观测局限，不作为语法边界正例或重编覆盖。

复现输入、class jar、反编译源码、Jarde 输出、编译/运行日志、`javap` 及其哈希均在本目录。工具版本和 Jarde 构建来源见 [tool-versions.txt](tool-versions.txt)。
