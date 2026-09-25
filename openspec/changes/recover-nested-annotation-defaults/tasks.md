## 1. 固定合法默认值与前置条件

- [x] 1.1 `annotation-default-boundaries/header-minimal/nested/` 固定普通 Java 8 Inner/Nested/runner 全类：原/JADX javac、`-Xverify:all` 反射输出 `6/2`；root 复制到 `/tmp/jarde-annotation-header-root-uD9gLJ` 重放，class hash 清单逐字节相同，Jarde 当前完整文本省略两个 default。现有两个非法 annotation 头部错误归 `spell-annotation-type-headers`，不是本项实现。
- [x] 1.2 在头部修复冻结 CLI 上确认 nested 类已可编译但反射默认值仍不同，作为本项独立修前执行基线；补空/多成员和一个子值无法拼写的受控边界，记录原 attribute 事实与完整类结果。修前完整类在 `/tmp/jarde-nested-default-before-2jwcd6r4` 编译成功，反射因 `Nested.child` 默认值为 null 而 NPE；原/JADX 输出均为 `6\n2\n`，fixture 的 `javap-Nested.txt` 保留两个 `AnnotationDefault`。

## 2. 私有默认值递归闭环

- [x] 2.1 在 `MemberDefault`/`resolve_default` 中递归解析 `ElementValueFacts::Annotation`，验证 descriptor 与成员名，保留元素顺序；不复制 reader parser，不扩普通字段 ConstantValue。
- [x] 2.2 用现有拼写/数组原子性输出标量及数组默认值，真实 class 单成员/多成员、空数组及不能拼写的后代分别验收，不输出半个 default。
- [x] 2.3 确认默认/完整来源正文相同、原始 class 身份及 reader 属性解析不变、预算/取消保持既有停止语义；不增加 class-source JSON 字段。相邻 B/C/I/J/S/Z、String、class、enum 和普通数组默认值回归不变。

## 3. 整类对照与主代理验收

- [x] 3.1 在已完成 `spell-annotation-type-headers` 的源码上，用完整 Engine/CLI 原样生成 Nested/Inner/runner，编译并反射运行，逐项等于原 class/JADX；不修改生成源码，也不把 F/D 或 enum 邻项纳入本项已支持声明。修后完整对照在 `/tmp/jarde-nested-default-final-b0_4znnc`，三套 `javac --release 8`、`java -Xverify:all` 均输出 `6\n2\n`。
- [x] 3.2 root 独立审查递归值/名称/数组失败边界，冻结重建 CLI 重放完整类；跑相邻 Rust/Java、reader census/fingerprint、fmt、Clippy、OpenSpec strict，记录既存债务。见 `verification.md`。
