## 1. 前置家族与三方分派证据

- [x] 1.1 核对 `assemble-proved-member-class-family` 的根/成员身份、捕获值与嵌套文本已由 Root 验收；重跑冻结 `OuterReceiverCases` 的原/JADX 四段 `20:10:1:3` 和 `javap` 的桥内 `invokespecial`，以 SHA 清单确认输入未变。Root 复核前置 [verification-5.1.md](../assemble-proved-member-class-family/verification-5.1.md)；`sha256sum -c` 的 31 项全通过，冻结 JAR SHA-256 为 `fee308350a667159e71cee7d8f6f9a73a9b63b0e3eab82f4de3554ecda0274a2`。从 JAR 重新 `javap -c -p` 确认 Member BCI 38 调用桥、桥 BCI 1 `invokespecial ReceiverBase.value:()I`；原 Java 和目标 JADX 家族均由 Root 以 `javac --release 8 -g:none` 重编，`java -Xverify:all` 各输出 `20:10:1:3`。JADX 中另一个 local class 是已冻结的不可编译负例，不纳入此正例。
- [x] 1.2 准备显式 `other` 接收者、不同直接父类目标、桥内额外效果/出口、第二调用者及 method handle/bootstrap 使用的 verifier-valid 或 proof-unit 负例；逐项记录实际物理引用与预期拒绝，不用不可验证的字节码推导行为。证据与覆盖边界见 [outer-super-bridge-negative](../../evidence/java-syntax-2026-09-27/outer-super-bridge-negative/README.md)。Root 重放 `run_evidence.py`，Java 8 编译、`-Xverify:all` 和 `javap -v -c -p` 全通过；显式 `other` 改写从 `120:10:11:10:10:3` 变为 `120:10:11:20:10:3`，直接父类目标变体输出 `102`。额外效果/出口及直接桥句柄只以 proof-unit 记录，未冒充可执行 class；反汇编已去除临时路径并二次重放确认哈希稳定。

## 2. 桥与使用闭包证明

- [x] 2.1 在选定 Outer 物理定义中证明唯一 synthetic/static 桥的准确 descriptor、单一 `invokespecial` 直接父类目标、参数映射及完整返回/异常路径；以正例和错误目标/额外效果负例测试拒绝。`prove_outer_super_bridge` 核对选定 Outer 与直接父类物理定义、逐指令加载/调用/返回及无 handler，证书保存准确目标 `PhysicalMethodId`；冻结正例和错误父类/静态目标/额外指令测试通过。此处是字节码证明，源级绑定仍归 3.1。
- [x] 2.2 对每个 Member 调用点将首参经 SSA 追到已证捕获 Outer，证明剩余实参的顺序和异常覆盖；用 `other: Outer` 与 Member 自身 `this.value()` 对照证明不能按类型或名字误投影。`prove_outer_super_call` 逐站点验证捕获字段的 SSA 值、实参依赖范围和 handler 边界；冻结同型 `other`/自身调用被拒，带两次 `tick` 效果的独立正例在调用 BCI 14 获得有序证书。
- [x] 2.3 在选定输入/可见依赖的完整结构引用范围内枚举桥的所有调用、method handle/bootstrap 和未解析使用，只有全部调用点被 2.2 覆盖才签发文本删除证书；以额外调用者、不透明引用、低预算与取消测试不发布部分闭包。`prove_outer_super_bridge_use_closure` 在目前可证明的单快照/单根闭包里查准确物理引用，要求结构/解析覆盖完整，逐调用产生证书；非 Member、constant/bootstrap、未决、低预算及取消均不能发布闭包。3.x writer 尚未消费证书，现有拒绝门继续阻止半成品文本。

## 3. 原子源码投影与来源

- [ ] 3.1 复用已证家族 writer 在 Member 体内呈现 `Outer.super.method(args)`，同时只从文本省略已闭合桥，保留物理 `ClassSourceMethod`；投影前证明 Java 8 重载、继承与泛型声明下生成调用仍绑定桥内准确目标 descriptor，必要时使用不改变行为的显式参数类型，无法证明则拒绝；以完整类 Java 8 编译、`-Xverify:all` 分派与效果对照以及错误目标拒绝验证。
- [ ] 3.2 给每处投影记录 Member caller BCI、Outer 桥 `invokespecial` BCI、准确物理目标及派生来源；以两个调用点共享一桥、根/child 同号 BCI、默认/all 模式和预算停止检查没有丢失或串错映射。

## 4. Root 独立验收

- [ ] 4.1 Root 从冻结 JAR 用重建 CLI 不编辑生成文本地比较原/JADX/Jarde 全类 Java 8 重编与四段运行，复核完整使用闭包及全部负例；运行相关 Rust 测试、`cargo fmt --all -- --check`、`git diff --check`、`openspec validate project-proved-outer-super-bridges --strict`，记录严格 Clippy/语料既有债务并清理隔离 Cargo target。
