## 1. 基线与负例

- [x] 1.1 重放固定 A1（SHA 核对）：javap 两形守卫块与 `<clinit>` 行精确形状；读守卫呈现位与隐藏挂点；记录基线。（SHA 与 `results/fixture-sha256.txt` 一致、两态基线复放一致；javap 取证、X=最外层类实测、呈现位/挂点阅读与判据落点见 [asg/README.md §1.1](../../evidence/java-syntax-2026-10-03/assert-stmt-patrol/asg/README.md)。）
- [x] 1.2 构造并冻结至少三个变体/负例：msg 方法调用副作用、嵌套类与外围同名字段、守卫额外语句（保持现呈现）；各自 `java -Xverify:all`（含 `-ea`）通过并记录实现前后行为。（正例 AssertProbe/A1 家族、负例 extra-statement/wrong-owner/MixedShapes 固化于 [asg/](../../evidence/java-syntax-2026-10-03/assert-stmt-patrol/asg/)；两态行为与负例基线腿逐字一致记录在案。）

## 2. 模式回写与消隐

- [x] 2.1 判据回写（design 决策 1–2）；A1 家族呈现 `assert` 语句、合成物消隐；两态行为一致。（判据落点 `crates/jarde-java/src/asserts.rs` + `src/facade.rs` 类级投影；A1 三处 `assert`、字段/clinit×2/守卫全消隐，重编两态逐字节一致。）
- [x] 2.2 负例保持；无 assert 类与既有 `<clinit>`/字段呈现 diff 逐字不变；预算/取消不变。（corpus 双腿 2165 类：2154 逐字一致、11 个 diff 全为 assert 家族形态；无 assert 类零变化；折叠 walk 按节点计费 IrItems、OutputBytes 先付后提交。）

## 3. 回归与验收

- [x] 3.1 全仓测试全绿、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。（`--tests` 口径 294 result 行全 ok 0 失败；`--all-targets --all-features` 口径 299 ok，两个失败 `p4_plugins` 与 `ordinary_generic_projection` 均为 handoff 记录 flake 家族、各复跑 3/3 绿；fmt 过；clippy 按 `ci.yml` 实有 29 项 `-A` 零警告；openspec strict 261/0 fail；fixture 计数与 corpus fingerprint 按流程重测更新。）
- [x] 3.2 A1 与变体三方对照：原 class（两态）/固定 JADX Java-input/Jarde 重编逐路径一致；记录输出 SHA。（见 [asg/results/README.md](../../evidence/java-syntax-2026-10-03/assert-stmt-patrol/asg/results/README.md)：两态逐字节一致、输出 SHA、corpus 双腿清单。）
- [x] 3.3 root 独立复核判据、消隐语境与两态行为，更新账本与巡查记录。（root 于合并主线 e227aa82 复核：A1 呈现三处 `assert`（两形+嵌套）、合成物零残留、两 `<clinit>` 整体省略；重编后**两态逐字节一致**——默认 `10/25/2/-2`、`-ea` `10/25/2`+`AssertionError: positive: -1`；全仓 2910/0、fmt/openspec 261/261。all-or-nothing census（Fieldref 物理操作闭合全字段）、wrong-owner 补丁类拒绝（最外层外围类校验）、与其它投影通道互锁保守拒绝且收紧到 assert 消隐类均复核认可。续作自纠三处 debug eprintln 残留——好例。）
