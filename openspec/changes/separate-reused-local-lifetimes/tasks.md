## 1. 冻结局部身份与控制流证据

- [x] 1.1 复用 `ReuseAfterForEach` 的 `-g`/`-g:none` class 与三方文本，补齐槽 2/3 的 SSA 定义、phi、逐读取值来源、Region 顺序和异常边核对；验证原/JADX 重编执行 `22 / 4 / NullPointerException`，冻结的预修复 Jarde 两份完整类均因 `int[]`/`int` 冲突而编译失败，并明确区别当前实现的成功重编结果。
- [x] 1.2 新增同类型连续赋值、跨分支或回边、异常处理器、category-2 槽相邻的最小 Java 8 夹具及 verifier/`javap` 记录；验证哪些可分段、哪些必须拒绝，不能以 LVT 名字或 BCI 大小当作唯一依据。

## 2. 局部身份与声明的最小闭环

- [x] 2.1 在既有声明类型计划中核对同一源局部所有将发射的写入，对确定的引用/原始类型冲突作判断；不能分段的 `int[]`→`int` 冲突须产出带 BCI 的未恢复原因，而非 `Recovered` 且不可编译的正文；验证冻结失败样本、1.2 拒绝边界、普通赋值和参数/资源头不回归；其他可赋值性问题拆出。
- [x] 2.2 扩展现有 `reuse::Plan`，按 SSA 定义/用途和 phi 连通性、Ref/Int 分类及 canonical CFG 正常边可达性证明 1.1 的两段生命周期；每次访问仅属一个分段，存在异常边或后段可返回前段时拒绝新推断；验证有无 LVT、1.2 的歧义负例、预算/取消和无半计划发布。
- [x] 2.3 让分段名称能分别携带有名或无名证据，并继续走 `NameTable` 的标识符、冲突与确定性规则；`-g` 后段保留 `first`、前段用独立名，`-g:none` 两段用稳定合成名；验证相同类型/既有双 LVT 分段、receiver、category-2 与 guard-header 规则不变。
- [x] 2.4 复用 `slot_uses`/`declarations` 分别定型和定作用域，完整类同时保留已证数组增强 `for` 与后继整数赋值；验证 `ReuseAfterForEach` 两份 Jarde 类和 runner 均通过 `javac --release 8`、`java -Xverify:all`，输出与原 class 一致，来源覆盖数组别名及后继写入 BCI。

## 3. 独立验收

- [x] 3.1 跑局部复用、声明、数组/Iterable 增强 `for`、异常/循环转移、预算和 source-map 定向回归；验证拒绝路径无类型矛盾的成功声明，essential/all 正文一致，`cargo fmt --all -- --check`、适当 Clippy、`git diff --check` 与 `openspec validate separate-reused-local-lifetimes --strict` 通过。
- [x] 3.2 root 用重新构建的 CLI 独立复验 1.1 的原/JADX/Jarde 完整类，以及 1.2 的原 class 与 Jarde 方法隔离 Java 8 编译、`-Xverify:all` 值/异常或明确拒绝及来源；审读 SSA/Region 分段证明和未覆盖债务，记录整仓既有门槛并清理独立 Cargo target。
