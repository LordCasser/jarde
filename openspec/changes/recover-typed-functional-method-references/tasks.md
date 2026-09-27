## 1. 固定基线与近邻

- [ ] 1.1 为已冻结 `TypedRefs.class` 写可重放脚本，核 class SHA、固定 JADX HEAD、原/JADX 的 Java 8 完整类重编、`-Xverify:all` 输出与三项反射泛型返回类型；脚本输出同时记录实施前 Jarde 的 raw/拒绝状态。
- [ ] 1.2 构造物理 descriptor 与直接 indy 保持不变但 `Signature` 参数或结果不匹配、正文额外效果/handler、nullable 或有副作用的绑定接收者近邻；证明各 class verifier 有效，冻结原始行为和当前安全边界。

## 2. 同轮函数目标证书

- [ ] 2.1 沿用现有 class-source Signature/erasure 路径，为精确 `Function<String,Integer>` 和 `Supplier<String>` 的完整直接返回方法建立受限目标事实；用静态、绑定和 Supplier 正例及伪造 Signature 负例核对目标不靠签名本身猜测。
- [ ] 2.2 将目标事实与同轮完整 Code/SSA/Program、唯一 LambdaMetafactory 站点、捕获来源和全部 BCI/CP 对齐，并接入现有普通参数化返回候选；用额外效果、异常行、非直接返回与预算/取消测试证明不发布半个候选。

## 3. 方法引用与声明共同发布

- [ ] 3.1 在受证参数化目标下区分 erased→instantiated 与 instantiated→implementation，只有 Java 8 `::` 可表达同一成员、参数及返回转换时才选择方法引用；用 `Integer::parseInt` 装箱、零参 Supplier 与现有 raw descriptor 适配回归核验。
- [ ] 3.2 对绑定形态只接纳入口 local 0 的当前实例及同一已解析成员，保留创建期捕获和调用对象身份；用 `this::length` 正例与 nullable、副作用、holder 替换近邻核验拒绝或原有行为。
- [ ] 3.3 在 class-source 现有 staging/checkpoint 中原子发布参数化声明与受证 AST 正文，并保留来源；用故意不匹配目标、输出预算耗尽和取消测试核对无 raw 头配 typed-only `::`、无有头无体或半份来源。

## 4. 三方验收与回归

- [ ] 4.1 用 fresh CLI 将原 class、pinned JADX 和 Jarde 的完整 Java 8 源码分别重编、`java -Xverify:all` 运行并逐字比较输出与反射泛型类型；核三种精确 `::`、每个工厂的物理 BCI/CP 来源及 verifier 有效负例。
- [ ] 4.2 跑固定非泛型 DT-27、`preserve-lambda-descriptor-adaptation`、数组构造引用、lambda helper 及相关 class-source 泛型回归，再跑 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、格式、OpenSpec strict 和 diff check；清理本任务专用 Cargo target。
- [ ] 4.3 root 独立审阅目标证书、函数适配、捕获阶段、负例与三方运行，更新 DT-27 清单并写验收记录；只验收此泛型切片，不宣称整个 DT-27 追平。
