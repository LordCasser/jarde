## 1. 固定整类与原始位拒绝边界

- [x] 1.1 冻结 `annotation-float-defaults/` 的自写 Java 8 注解类、runner、class SHA/293B、`javap -v -c -p` 与原/JADX/Jarde 完整类；root 从复制目录独立重放，原/JADX 四行 raw bits 相同，Jarde 全类能编译但反射 NPE，确认是 F/D 默认值单因子缺口。
- [x] 1.2 固定单池项修改的负 quiet NaN 与 payload NaN JVM 合法 class，运行 `-Xverify:all` 记录原始 raw bits、JADX/Jarde 结果和成员表未变；另测数组 F/D 一处无法拼写时整段拒绝，不把 patch 结果冒充普通 Java 源码。

## 2. 现有默认值路径的最小扩展

- [x] 2.1 在 `MemberDefault`/`resolve_default` 增加仅供 AnnotationDefault 的 F/D 原始 bits，严格匹配 tag 与池项；字段 ConstantValue 路径不扩张，结构化报告原始事实不变。
- [x] 2.2 在已有 emitter 规则内为有限值提供精确 hex 拼写，保留负零、次正规、端点和 f/d 类型；标准无穷及正 quiet NaN 用经 Java 8 验证的常量表达式，其它 NaN 位模式拒绝。
- [x] 2.3 复用数组全有或全无递归；验证默认/完整 evidence 文本一致、来源事实与预算/取消不变，并跑相邻普通默认值及嵌套注解回归。

## 3. 执行对照与主代理验收

- [x] 3.1 用完整 Engine/CLI 原样生成 1.1 的完整类，javac Java 8 编译、`java -Xverify:all` 反射四行逐项等于原/JADX；补有限边界、正负 Infinity、NaN 拒绝和 F/D 数组的真实执行，不手改生成文本。
- [x] 3.2 root 独立审查位拼写、特殊值准入、字段/数组拒绝来源，冻结重建 CLI 重放整类和 patch；运行受影响 Rust/Java、reader census/fingerprint、fmt、Clippy、OpenSpec strict，记录既存独立门禁债务。结果见 `verification.md` 及 `annotation-float-defaults/post-fix-root-replay/`。
