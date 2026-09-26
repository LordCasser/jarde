## 1. 冻结三方基线

- [x] 1.1 编译冻结 `static-member-basic` 的两个 Java 8 源文件，记录三份 class SHA、反汇编及原/JADX/Jarde 完整源码；执行 `python3 openspec/evidence/java-syntax-2026-09-27/static-member-basic/replay.py`，三方无原 jar classpath 重编和 `-Xverify:all` 均输出 `9:4`，旧 Jarde 保留独立 `$Leaf` 源形。
- [ ] 1.2 加双方成员行冲突、缺子类/同名多定义、第二直接子类、正文 fallback 和低预算/取消负例；定向测试逐项证明根类在证据不全时不发布半个嵌套类，同时 `Named$Top` 保持顶级身份。

## 2. 静态家族关系与正文证明

- [ ] 2.1 让现有 typed 成员候选接纳准确静态子类，同时区分静态无捕获与非静态捕获证明状态；用 `static-member-basic` 和现有非静态成员家族测试核对根子双方关系、唯一性与非静态回归。
- [ ] 2.2 证明无字段、非泛型、唯一平凡无参构造器及子类全部需呈现方法体的完整性；用构造器加效果/字段/混合质量负例验证拒绝，不从源码文本或 `contains_statements` 推断完整。

## 3. 源码单元原子投影

- [ ] 3.1 在已证目标的同次方法 AST 和描述符拼写中，把根类的直接无参创建与返回类型写成成员源名，保留语义二进制类型；定向测试同时断言 `new Leaf()`、`Leaf make()` 与 `Named$Top` 不变。
- [ ] 3.2 共用根类 writer 装配 `static class Leaf`、正确源名构造器和全部方法，并保留独立物理报告/来源；任一重发射或预算失败不提交局部文字。以根类单元加真实独立依赖执行 Java 8 重编和 `-Xverify:all`，输出与原 class 一致。

## 4. 架构师独立验收

- [ ] 4.1 Root 独立复跑冻结三方对照、定向和非静态家族回归，核查静态身份/预算/来源，执行 `cargo fmt --all -- --check`、`openspec validate present-proved-static-member-class --strict`；通过后更新 DT-02 状态并提交推送。
