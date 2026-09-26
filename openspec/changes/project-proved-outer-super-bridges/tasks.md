## 1. 前置家族与三方分派证据

- [ ] 1.1 核对 `assemble-proved-member-class-family` 的根/成员身份、捕获值与嵌套文本已由 Root 验收；重跑冻结 `OuterReceiverCases` 的原/JADX 四段 `20:10:1:3` 和 `javap` 的桥内 `invokespecial`，以 SHA 清单确认输入未变。
- [ ] 1.2 准备显式 `other` 接收者、不同直接父类目标、桥内额外效果/出口、第二调用者及 method handle/bootstrap 使用的 verifier-valid 或 proof-unit 负例；逐项记录实际物理引用与预期拒绝，不用不可验证的字节码推导行为。

## 2. 桥与使用闭包证明

- [ ] 2.1 在选定 Outer 物理定义中证明唯一 synthetic/static 桥的准确 descriptor、单一 `invokespecial` 直接父类目标、参数映射及完整返回/异常路径；以正例和错误目标/额外效果负例测试拒绝。
- [ ] 2.2 对每个 Member 调用点将首参经 SSA 追到已证捕获 Outer，证明剩余实参的顺序和异常覆盖；用 `other: Outer` 与 Member 自身 `this.value()` 对照证明不能按类型或名字误投影。
- [ ] 2.3 在选定输入/可见依赖的完整结构引用范围内枚举桥的所有调用、method handle/bootstrap 和未解析使用，只有全部调用点被 2.2 覆盖才签发文本删除证书；以额外调用者、不透明引用、低预算与取消测试不发布部分闭包。

## 3. 原子源码投影与来源

- [ ] 3.1 复用已证家族 writer 在 Member 体内呈现 `Outer.super.method(args)`，同时只从文本省略已闭合桥，保留物理 `ClassSourceMethod`；以完整类 Java 8 编译、`-Xverify:all` 分派与效果对照以及错误目标拒绝验证。
- [ ] 3.2 给每处投影记录 Member caller BCI、Outer 桥 `invokespecial` BCI、准确物理目标及派生来源；以两个调用点共享一桥、根/child 同号 BCI、默认/all 模式和预算停止检查没有丢失或串错映射。

## 4. Root 独立验收

- [ ] 4.1 Root 从冻结 JAR 用重建 CLI 不编辑生成文本地比较原/JADX/Jarde 全类 Java 8 重编与四段运行，复核完整使用闭包及全部负例；运行相关 Rust 测试、`cargo fmt --all -- --check`、`git diff --check`、`openspec validate project-proved-outer-super-bridges --strict`，记录严格 Clippy/语料既有债务并清理隔离 Cargo target。
