## 1. 固定完整类与拒绝边界

- [x] 1.1 用自写 Java 8 `@Deprecated` 类、`@Retention(RUNTIME)` 注解类型及 runner 冻结完整 class/`javap -v -c -p`/JADX/Jarde 文本和 JSON；root 在复制目录独立重放，三套源码均编译、`-Xverify:all` 执行，原/JADX 三行相同而 Jarde 为 `false`、`3`、`null`，源码/class SHA 在 `class-annotation-uses/generated/summary.json`。
- [x] 1.2 增加 CLASS-retention 的 `RuntimeInvisibleAnnotations` 正面类与不能完整拼写的单条注解、同类型重复、属性内容损坏/预算拒绝受控边界；逐项记录 class SHA、物理属性跨度、原行为或拒绝来源，不把 `Deprecated` 标记属性误当注解。验证脚本重放、reader 定向测试与原 class 反射/`javap` 对照。

## 2. 最小类属性解析及源码装配

- [x] 2.1 在现有 reader 属性读取中复用 `element_value` 读树解析类级 visible/invisible 两属性，保持 tag/池索引/顺序/深度和精确末端；验证 reader 正负测试、预算与取消，且无属性时无额外内容读取。
- [x] 2.2 在 `Engine::class_source` 同一次 `read.facts`/惰性池路径读取类属性一次，将属性壳与已解析事实交给声明装配；验证一类一次 materialize、无 body 的 annotation type、类头及 member item 原始事实/JSON 仍可核对。
- [x] 2.3 复用注解默认值的受限值拼写与标识符规则，完整注解按条提交；类源码和 JSON 同时呈现拼写及拒绝，跨属性按物理顺序、组内按字节顺序，重复类型保守拒绝；验证 marker/枚举值、具名数组和不能拼写值的 Rust 定向测试与默认/完整 evidence 文本相同。
- [x] 2.4 验证属性损坏、预算/取消、同类型重复和非法名字的结果不伪装成功；检查普通无注解类、字段/方法/参数/类型使用的源码及方法 IR/AST 回归无变化。

## 3. 三方执行与主代理验收

- [x] 3.1 用修后完整 Engine/CLI 原样重放 1.1/1.2，原/JADX/Jarde 各自 Java 8 编译及 `java -Xverify:all`，反射逐项相同；`RuntimeInvisibleAnnotations` 以原/修后 class 属性类型和值对照，不把运行时不可见当作可见。
- [x] 3.2 root 独立审查类属性来源、按需计费、原子拒绝、JSON/文本一致和外部类型边界；冻结重建 CLI，在复制目录重放全部证据，运行相关 Rust/Java、reader census/fingerprint、fmt、Clippy、OpenSpec strict，并将独立门禁债务单列。逐项结果见 `verification.md`。
