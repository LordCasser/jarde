# 命名成员类词法外层接收者：分派目标证据

日期：2026-09-26。Fixture 按 Java 8 编译。`Outer extends Base`，`Base.value()` 返回 `outer-base`，`Outer.value()` 覆写为 `outer-override`；非静态命名成员类 `Outer.Member extends MemberBase`，`MemberBase.value()` 返回 `member-base`。因此 `Outer.super.value()` 必须选 `Base.value()`（绕过 Outer 覆写），普通 `this.value()` 必须选 `MemberBase.value()`。这是用不同的可观测结果区分词法外层接收者与当前对象接收者。

## 可直接证明的事实

- 原源码以 `javac --release 8 -g` 编译成功并运行输出 `7:outer:outer-base:member-base`。`original-run.txt` 是重编译类的运行结果。
- `Outer$Member.class` 含 `final synthetic Outer this$0`。`outerField`、`outerMethod`、`outerSuper` 均先读取该字段，再调用 Outer 上的 `access$...` 桥；普通 `ordinaryThis` 使用当前接收者的 `invokevirtual value()`。见 `javap-Outer-Member.txt` 与 `javap-key-shapes.txt`。
- `Outer.class` 的 `access$201(Outer)` 内部以 `invokespecial Base.value()` 实现限定外层 super；访问外层 private 字段/方法另有 `access$000/100`。见 `javap-Outer.txt` 与 `javap-Outer-key-shapes.txt`。结合上面的独立返回值，这直接确认了桥接目标，而非根据桥名猜测。
- JADX 1.5.6 对完整 jar 的反编译保留了 `Outer.super.value()`，普通调用写成未限定 `value()`（处于 `MemberBase` 继承上下文）。全量源码以 `--release 8` 重编译并运行，输出仍是 `7:outer:outer-base:member-base`。因此此区分 fixture 下，JADX 限定名保持了目标分派；不会出现此前父类相同导致裸 `super` 偶然输出一致的歧义。全量源码见 `jadx-full/sources/defpackage/`。单独反编译 `Outer$Member.class` 时仍能看到 `access$000/100/201` 桥接调用，见 `jadx/`。
- Jarde CLI 从干净 `git archive e9bb60bb41aeeab241fc78bef348f7564b74fcd3` 独立构建，未读取工作树中的未提交 `build.rs`。`class-source --class 'Outer$Member'` 输出 `Outer$Member extends MemberBase`，三个词法外层访问仍通过 `Outer.access$...`，普通调用为 `this.value()`。完整输出为 `jarde-Outer-Member.java`。其单类文本明示不是可编译项目；独立 javac 记录在 `jarde-javac.txt`，它缺少 Outer、Base 等项目声明且不是完整词法嵌套源码，故此处不把单类编译失败等同于 Jarde 的完整源码编译失败。

## 边界与推断

此 fixture 证明：`Outer.super` 的 dispatch receiver 是捕获的 Outer 实例，目标为 Outer 的直接父类 Base；普通 `this` 是 Member 实例，目标由 MemberBase 继承关系确定。恢复外层接收者需要同时证明词法 owner、捕获实例的数据流、桥接目标和 Java 声明关系。不能由 `this$0` 或 `access$` 命名本身推导目标。

探针只记录事实，不提出恢复方案，不修改生产代码或 OpenSpec。`special_receiver` 对当前入口 this/直接父类型的判断尚不覆盖这里的捕获外层对象证明。

## 复现材料

- 输入源码：`Outer.java`、`Base.java`、`MemberBase.java`
- class 与源码/class SHA-256：`classes/*.class`、`sha256.txt`
- javap 完整输出与关键行：`javap-Outer-Member.txt`、`javap-Outer.txt`、`javap-key-shapes.txt`、`javap-Outer-key-shapes.txt`
- 原源码/JADX/Jarde 输出：`original-run.txt`、`jadx-full/`、`jadx/`、`jarde-Outer-Member.java`
- 编译及 Jarde 报告：`original-javac.txt`、`jadx-javac.txt`、`jarde-javac.txt`、`jarde-stderr.txt`
- 工具与 baseline：`tool-versions.txt`
