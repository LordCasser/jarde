# DT-02：静态成员类与顶级 `$` 对照

输入冻结在 `tests/fixtures/proved-java-structure/static-member-basic/`，由 JDK 23.0.1 的 `javac --release 8 -g:none` 编译，三份 class 哈希和 `javap -p -c -s` 一并保存。`StaticMemberBasic$Leaf` 是带双方 `InnerClasses` 关系的静态成员，`Named$Top` 则是仅名字含 `$` 的独立顶级类。原始程序输出 `9:4`。

本地 JADX 1.5.6 把前者投影为 `StaticMemberBasic` 内的 `static class Leaf`，在 `make()` 中写 `new Leaf()`；对后者保留独立 `class Named$Top`。Jarde 当前将前者作为独立的 `class StaticMemberBasic$Leaf`，并在 `make()` 中保留 `new StaticMemberBasic$Leaf()`。这是一项**源码形态差距**；原/JADX/Jarde 的各自完整源码集合都通过不依赖原 jar 的 Java 8 重编及 `java -Xverify:all`，三方输出同为 `9:4`。相应源码和日志在本目录。

Jarde 的 `src/member_inner.rs::scan_family_root` 明确只接纳非静态具名成员；已有的 `ClassSourceMemberFamily` 投影还要求外部实例捕获。DT-02 应复用 typed 双方成员行、选定物理定义、完整子类方法及家族源码装配，但静态成员不能伪造捕获证书。`Named$Top` 是必需的拒绝控制；不能按 `$` 字符串替换全局改名。如何推广家族装配要在独立 OpenSpec 中确定，不能混进 DT-05 匿名接口的实现。

运行 `python3 replay.py` 重放冻结 class 身份、原/JADX/Jarde 三方编译执行。脚本使用临时 Cargo target 并在退出时删除构建及中间目录。
