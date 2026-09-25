# 匿名类跨类使用：受控证据

日期：2026-09-25。环境：JDK（目标为 Java 8）、JADX 1.5.6（`/Users/lordcasser/workspace/testzone/jadx`）、本工作树构建的 Jarde CLI。没有修改生产代码或 OpenSpec 变更计划文件。

## 输入与受控变异

[fixture 源码](../../../../tests/fixtures/proved-java-structure/anonymous-cross-class-use/) 定义了 `Base`、`Owner.one()`、`Other.two()` 和 `Main`。两个静态工厂各自创建一个匿名 `Base` 子类，避免捕获外部实例，所以两个生成构造器描述符均为 `()V`。

运行 `python3 tests/fixtures/proved-java-structure/anonymous-cross-class-use/freeze.py`：脚本用 `javac --release 8 -g:none` 编译未修改源码，原始类在 `java -Xverify:all` 下输出 `sameClass=false`；随后只改 `Other.class` 常量池中唯一的等长 UTF8 `Other$1` → `Owner$1`，保留两个匿名类文件。变异类集通过 `java -Xverify:all` 并输出 `sameClass=true`。真实输出见 [freeze.log](freeze.log)，冻结 class 哈希见 [class-sha256.log](class-sha256.log)。`Owner$1.class`、`Other$1.class`、变异后的 `Other.class` 哈希分别为 `377750e724cbe67b3c7b9a8518a0d7aa92448ba9fcd7e6625aa333d9dde797b8`、`0d5cef597b73f5ba3581a425511c4f9dc818d004e3ba658cbc7aad575bc46e45`、`258f29995f016d966cfaa3fa5e225a57e096d84b80420a2a2e94105a4f8d7210`。`javap -v -p Owner Other 'Owner$1' 'Other$1'` 的结果在 [javap.log](javap.log)，可核对两匿名类仍各自存在、构造器均为 `()V`，且变异后 `Other.two()` 的 `new`/`invokespecial` 均指向 `Owner$1`。

## JADX 1.5.6 全 jar 反编译和重编

重放脚本把冻结 class 打成 jar，运行 `jadx -d <临时输出目录> <input.jar>`。JADX 报告完成并生成 `Base.java`、`Main.java`、`Other.java`、`Owner.java`；文件清单见 [jadx-files.log](jadx-files.log)，本次实际生成源码保存在 [jadx-source](jadx-source/)。

全量执行 `javac --release 8 -d <输出目录> <全部生成的 Java 文件>` 失败，详情见 [jadx-javac.log](jadx-javac.log)。可直接在 [Owner.java 第 21–22 行](jadx-source/Owner.java) 和 [Other.java 第 21–22 行](jadx-source/Other.java) 复核：JADX 将 `Owner$1`、`Other$1` 生成为各自 owner 内的非静态 `AnonymousClass1`；静态 `Owner.one()` 因隐式 `this` 编译失败，`Other.two()` 则尝试无外部 `Owner` 实例地创建 `Owner.AnonymousClass1`。因此虽完成全 jar 源码生成，生成项目不能重编；没有从 JADX 源码执行语义对照。

## Jarde class-source 和带 classpath 重编

重放时从同一个冻结 jar 分别执行：

```
jarde-cli class-source --input <input.jar> --class Owner --policy plain-jar --release 8 --format text
jarde-cli class-source --input <input.jar> --class Other --policy plain-jar --release 8 --format text
```

输出全文分别保存在 [jarde-Owner.java](jarde-Owner.java) 和 [jarde-Other.java](jarde-Other.java)，两份方法体都呈现 `return new Owner$1();`。这忠实暴露了变异 `Other.class` 的跨类目标；但类级输出不包含 `Base` 或任一匿名类声明，且文本头部明确说明它不承诺可编译。

为避免缺少 `Base` 或兄弟 class 干扰判断，分别以正确文件名执行 `javac --release 8 -cp <input.jar> -d <输出目录> <Owner.java|Other.java>`。两份源码**单独**编译都成功（[Owner 日志](jarde-Owner-javac.log)、[Other 日志](jarde-Other-javac.log)）；分别把新 `Owner.class` 或新 `Other.class` 置于原 jar 前运行，都仍得 `sameClass=true`（[Owner 运行](jarde-Owner-run.log)、[Other 运行](jarde-Other-run.log)）。单独编译时，原 jar 里的 `Owner.class` 和 `Owner$1.class` 提供了交叉引用，因此单编成功不代表生成的源码集合自身完整。

将 `Owner.java` 与 `Other.java` **一起**以相同 classpath 编译时，`Other.java` 的 `return new Owner$1();` 找不到符号 `Owner$1`（[日志](jarde-combined-javac.log)）。因此 Jarde 的两份类级报告还不能组成可编译的 Java 源码集合；这是更严格的证据边界，不能把单编成功写成全量提取成功，也不能把全量失败误归因于缺少 `Base`。

## 重放

运行 `python3 openspec/evidence/java-syntax-2026-09-25/anonymous-cross-class-use/replay.py` 可重做 jar 构造、JADX 输出与重编、Jarde 两类输出与带 classpath 重编。脚本在临时目录以私有 `CARGO_TARGET_DIR` 构建 CLI，退出时清理该目录；输出日志、JADX 小型源码副本和 Jarde 源码会刷新。冻结源码基线与常量池变异单独由 `freeze.py` 重放。
