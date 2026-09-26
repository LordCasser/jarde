## 1. 冻结三方基线

- [x] 1.1 编译冻结 `static-member-basic` 的两个 Java 8 源文件，记录三份 class SHA、反汇编及原/JADX/Jarde 完整源码；执行 `python3 openspec/evidence/java-syntax-2026-09-27/static-member-basic/replay.py`，三方无原 jar classpath 重编和 `-Xverify:all` 均输出 `9:4`，旧 Jarde 保留独立 `$Leaf` 源形。
- [x] 1.2 加 root/child typed 成员行冲突、缺子类/同名多定义、第二直接静态子类和低预算/取消负例；定向测试逐项证明根类在这些证据缺失或冲突时不发布半个嵌套类，同时 `Named$Top` 保持顶级身份。

## 2. 静态家族关系与正文证明

- [x] 2.1 让现有 typed 成员候选接纳准确静态子类，同时区分静态无捕获与非静态捕获证明状态；用 `static-member-basic` 和现有非静态成员家族测试核对根子双方关系、唯一性与非静态回归。
- [x] 2.2 静态目标需满足无字段、非泛型、唯一平凡无参构造器及子类全部方法完整结构化的发布门；字段与构造器副作用负例验证目标拒绝，不从源码文本或 `contains_statements` 推断完整。静态专属 fallback/mixed-quality fixture 不在此最小切片内。

## 3. 源码单元原子投影

- [x] 3.1 用同次 AST 候选和准确 descriptor/BCI 证明根类直接无参创建及返回类型；恢复阶段保留物理 `$` 拼写，最终家族 writer 才提交 `new Leaf()` 与 `Leaf make()`，并保持 `Named$Top` 顶级身份。
- [x] 3.2 共用根类 writer 装配 `static class Leaf`、正确源名构造器和全部方法，并保留独立物理报告/来源；完整重发射及输出预算通过后一次性提交。预算拒绝测试断言完整物理 root 仍含 `StaticMemberBasic$Leaf make()` 与 `new StaticMemberBasic$Leaf()`；额外 `echo(Leaf)` 参数/返回/强转负例不能发布仍含物理类型的嵌套源码。以根类单元加真实独立依赖执行 Java 8 重编和 `-Xverify:all`，输出与原 class 一致。

## 4. 架构师独立验收

- [x] 4.1 Root 独立复跑冻结三方对照、定向和非静态家族回归，核查静态身份/预算/来源，执行 `cargo fmt --all -- --check`、`openspec validate present-proved-static-member-class --strict`；通过后更新 DT-02 状态并提交推送。
