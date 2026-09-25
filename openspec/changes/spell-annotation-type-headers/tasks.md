## 1. 固定头部单因子证据

- [x] 1.1 固定 `annotation-default-boundaries/header-minimal/basic/` 普通 Java 8 注解与反射 runner：原/JADX 完整类编译验证运行相同，修前 Jarde 完整类唯一错误是显式 `extends Annotation`。root 复制到 `/tmp/jarde-annotation-header-root-uD9gLJ` 独立重放，class 哈希清单逐字节相同，三行原/JADX输出一致，Jarde唯一头部编译错误重现。
- [x] 1.2 固定 `nested/` 对照：两类修前 Jarde 仅有两个头部编译错误，同时 `child/children` 默认值静默缺失；root 复制重放两类 hash、原/JADX `6/2` 反射值及未改 Jarde 文本。该默认值缺口归独立 change，不纳入头部实现。

## 2. 最小拼写修复

- [x] 2.1 在现有 `class_declaration` 对 canonical annotation 接口表使用隐式注解头，不将 `java/lang/annotation/Annotation` 发射成 `extends`；结构化报告保留原始 flags/接口表，无新机制。
- [x] 2.2 更新现有 `p3_annotation_default` 类头断言，并验证普通接口继承、普通类实现、非 canonical 注解接口表的保守边界及预算/默认来源请求正文稳定。

## 3. 整类执行与主代理验收

- [x] 3.1 用完整 Engine/CLI 原样生成 Basic 全类并编译/反射执行，逐行等于原 class/JADX；复核 nested 头部合法但 default 缺失仍作为独立 RED，不修改生成文本掩盖它。
- [x] 3.2 root 重建冻结 CLI 独立审查 class 头与 item 事实、完整类对照，跑相邻 Rust/Java 测试、reader census/fingerprint、fmt、Clippy、OpenSpec strict，分开记录既存门禁债务。见 `verification.md`。
