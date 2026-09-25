## 1. 冻结三方基线和反证

- [x] 1.1 以 `enum-switch-labels/` 的 Java 8 原类、source-only runner 和合成 helper 为固定输入，记录 class/JADX/Jarde 完整源码、`javap`、SHA、javac 与 `-Xverify:all` 四行输出；`replay.py` 在独立目录复编四个 class 与冻结 SHA 逐字节相同，`replay-baseline/summary.json` 记录 CLI SHA `6e2d1a3ee7c18622d98c0695ecc87bbaaad03acecc8795c1d17567aa6330859f`。原与 JADX 均编译执行 `1|1、2|2、3|3、null|0`；Jarde 完整类输出整数表 switch，哪怕提供 helper class 仍因 synthetic `$SwitchMap$Hue` 不可源码引用而 javac exit 1。
- [ ] 1.2 原样重放仅交换 helper `<clinit>` BCI 18/33 常量的合法补丁，固定原 class `2|2、1|1、3|3、null|0` 和 JADX 的交换标签重编结果；另加真实多写/非唯一键/缺失依赖边界，并加入 `negative/aliased-enum` 字段 alias 与 `negative/enum-values-array` 中 null、短数组或额外效果的 verifier-valid 反例，确认验证器状态与错误 direct-switch 结果，供拒绝投影测试使用。
- [ ] 1.3 将必要 subject/helper/enum class、源码和边界加入永久 fixture，独立重编核对哈希、reader census 与 corpus fingerprint；只冻结输入，不把 JADX 源码或旧 Jarde 文本当产品能力。

## 2. 复用同次结论证明跨类映射

- [ ] 2.1 从当前 `enumswitch@1` 计划给 class-source 同次候选带出表 Fieldref、索引调用、selector 与 switch BCI/SSA 身份，不解读已发射正文；单方法路径不引入候选。定向测试核对原/补丁方法身份相同、候选目标相同及 essential/all 判定一致。
- [ ] 2.2 使用现有环境/reader 按需解析表定义与 enum 类型，证明 helper/字段/常量唯一、真实 `ACC_ENUM` 与分派调用关系；进一步用 enum `<clinit>`、`(String,int)` 构造器、`<clinit>` 所调用的唯一数组工厂和公开 `values()` 的完整 IR，证明每个常量字段独立初始化、`java/lang/Enum` 收到字段名和从 0 连续的 ordinal、数组工厂按 ordinal 装入同一数组且 `values()` 只克隆已发布数组。alias、重复/稀疏 ordinal、数组长度/内容/返回路径改写、缺失字段和未知形状拒绝；传入的每份 IR 都须被证明逻辑实际消费。SingleClass、缺失、歧义和错误 owner 均拒绝，预算/读取计数测试证明不遍历无关类。私有工厂与公开 `values()` 的双 IR 证明、getstatic BCI 身份及两份 verifier-valid 数组负例已通过[独立复核](../../evidence/java-syntax-2026-09-24/enum-switch-labels/values-proof-review.md)；其余依赖/预算/读取计数门槛尚未逐项验收，故不勾选。
- [ ] 2.3 从 helper 的真实 `<clinit>` Code/CFG/SSA 和异常表证明整张表的分配、唯一写入、key→常量、`NoSuchFieldError` 处理器及无额外效果；交换补丁必须导出相反映射，重复/遗漏写入和其他可见改写必须拒绝，定向 Rust 测试核对每条 BCI 与拒绝原因。

## 3. 投影、预算与验收

- [ ] 3.1 只对完整证明的 class-source 方法，用同次 AST sidecar 原子投影 `switch(enum)` 与 enum case；独立方法的原整数分派及完整类物理方法/原 `RecoveryReport` 保持可查。正常和交换映射两份完整类源码分别以 `javac --release 8` 重编、`java -Xverify:all` 输出与各自原 class 四行一致，null 与 trace 逐项相同。
- [ ] 3.2 对 sidecar、依赖读取、映射证明、来源和第二次文本发射逐项计费并轮询取消；低依赖/IR/输出预算或取消时整项无半投影，essential/all 正文一致，定向库/CLI 测试检验停止状态与原物理证据。
- [ ] 3.3 root 独立审读枚举身份、真实表映射与异常/副作用路径，在独立目录重放原/JADX/Jarde、合法交换与拒绝边界；运行相邻 enum-switch、switch-expression、class-source、reader census/fingerprint、fmt、严格 Clippy 和 `openspec validate project-proved-enum-switch-labels --strict`，只在证据满足后勾选。
