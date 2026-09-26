# 命名成员类家族第一阶段证据

Primary fixture 只包含顶层 `NamedMemberFamilyStage1` 与其命名非静态成员类 `Member`，不含 static nested 或 local class。`javac --release 8 -g` 编译成 `fixture.jar` 后，原版由 `java -Xverify:all` 执行，输出 `2011` 和 `20`。成员方法将显式参数 `other.state`（20）与捕获的词法外层私有字段 `NamedMemberFamilyStage1.this.secret`（11）编码进结果 `2011`；main 另外直接读 `other.state` 输出 20。

[完整 javap 记录](javap-all.txt) 显示 `Member` 只有一个物理构造器 `<init>(NamedMemberFamilyStage1)`，一个 `this$0` 捕获字段；它分别从 `other` 直接读 `state`，并通过唯一的 `access$000(this$0)` getter bridge 读外层 `secret`。本 fixture 没有 `Outer.super`、`access$101` 或静态嵌套声明。原始输入、class 与 JADX 文本的 SHA-256 见 [sha256-original.txt](sha256-original.txt)；`.class` 哈希在删除重复编译输出前生成，原字节保留在唯一的 `fixture.jar` 中。

JADX 1.5.6 对同一 JAR 生成 [完整类源码](jadx/sources/defpackage/NamedMemberFamilyStage1.java)。原样 `package defpackage;` 没有被编辑：直接将该文件传给 `javac --release 8` 成功，并用包限定名 `defpackage.NamedMemberFamilyStage1` 在 `-Xverify:all` 下运行，输出同为 `2011`、`20`。命令状态与结果在 `jadx-raw-javac.*`、`jadx-raw-run.*`。另保留去掉 JADX 添加的包声明后的编译对照，结果在 `recompiled-jadx/`、`jadx-javac.*` 与 `jadx-run.*`。两次重编的 class 文件运行验证后都从证据目录移除，避免重复保存 JAR 字节。

static nested 与 local class 是独立边界，不属于本正例。静态嵌套沿用相邻目录的 [`OuterReceiverCases.java`](../named-member-outer-receiver/variants/OuterReceiverCases.java)、[`jarde-StaticNested.java`](../named-member-outer-receiver/variants/jarde-StaticNested.java) 及既有非法 `Outer.this` / `Outer.super` 对照 [`StaticNestedIllegal.java`](../named-member-outer-receiver/variants/StaticNestedIllegal.java)。local class 对照沿用 [`OuterLocalBoundary.java`](../named-member-outer-receiver/variants/OuterLocalBoundary.java)、[`javap-Local.txt`](../named-member-outer-receiver/variants/javap-Local.txt)、[`local-original-run.txt`](../named-member-outer-receiver/variants/local-original-run.txt) 与 [`jarde-Local.java`](../named-member-outer-receiver/variants/jarde-Local.java)；其 `EnclosingMethod`、`val$other` 和 `this$0` 显示它与命名成员声明的身份边界不同。

Jarde 对照从 `git archive 9406e758` 的固定源码另建 CLI，二进制 SHA-256、构建与请求参数见 [`jarde/build.txt`](jarde/build.txt)。完整 JSON 和根/成员两份原样文本位于 [`jarde/`](jarde/)。报告的 `member_family` 为 `prepared`，根有 3 个物理方法、成员有 2 个；两份文本仍分别声明物理类，不代表已完成源码家族投影。单独编译根文本成功，但 `java -Xverify:all` 只输出 `20`，缺失成员构造与调用的 `2011`；将根和成员文本一起交给 `javac --release 8` 则因成员构造器把 `this.this$0 = this$0` 写在 `super()` 前而失败。完整编译诊断和退出码也保存在 [`jarde/`](jarde/)。这两个结果精确描述身份阶段之后仍待 3.x/4.x 证明和投影的缺口。

工具与运行状态见 [tool-versions.txt](tool-versions.txt) 和 [status.txt](status.txt)；本目录文件哈希见 [sha256-evidence.txt](sha256-evidence.txt)。
