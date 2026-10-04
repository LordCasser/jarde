## 1. 取证与冻结

- [ ] 1.1 复核 proposal 的五行落点表（`facade.rs` 14094 / 15906 / 15920 / 15923 / 15928，行号随主线漂移、以锚点名为准），并确认数据通路缺口：`PendingEnumConstructorEdge`（约 13839）只有 `caller`/`call_bci`/`target_owner`/`target_descriptor` 四项，**不携带字段名**；14094 处 `EnumCodeReference::Field { name, … }` 的 `name` 即已证名但被 `matches!` 丢弃。
- [ ] 1.2 重放六形对照（[证据](../../evidence/java-syntax-2026-10-04/enum-string-field-name-hardcode/)）：确认判别变量是字段名本身（C1/C3 的 `op` 投影成功，C2/C4/C5/C6 的 `t`/`value`/`x` 不投影），且 C1 渲染源集 `javac --release 8` **exit 0**、C2 渲染源集 **exit 1 `此处需要枚举常量`**（root 已实测，须复现）。
- [ ] 1.3 冻结**非 `op` 字段名的正例** fixture（源级，如字段名 `label` 或 `t`，两常量各带 ASCII String 实参与常量专属体），并确认它在当前主线上**不投影**（作为本片的红灯锚）。**该正例必须有 CI 测试引用**（handoff 强制纪律），否则缺陷无法被 CI 发现——这也是 `recover-proved-string-arg-enum-constant-bodies` 当初漏掉的原因。
- [ ] 1.4 冻结负例：两个私有 String 字段且写入目标不唯一；构造器 Code 不完整；`putfield` 目标非 String 描述符。

## 2. 实现

- [ ] 2.1 打通"已证字段名 → 发射处"的通路：在 `PendingEnumConstructorEdge`（或等价证明载体）上增加已证字段名字段，由 14094 处 `putfield` 的 `name` 绑定填充；**不得**在发射处重新猜测或回退到字面量。
- [ ] 2.2 把 15906/15920/15923 三处的 `== b"op"` 判据改为"**被该构造器 `putfield` 写入的 String 字段**恰一个"，其余既有属性检查（描述符、owner、非 static/synthetic/隐式枚举成员、`ACC_PRIVATE`、有 `declaration`、无 markers）**逐字保留**。
- [ ] 2.3 把 15928 的发射文本 `this.op = arg0;` 改为用已证字段名拼写；14094 的错误文本 `"the String constructor does not preserve Enum and op semantics"` 中的 `op` 措辞同步改为不依赖固定名（该文本可能出现在既有断言里，须核实并如实更新，不得为让旧断言变绿而削弱判据）。
- [ ] 2.4 **不越界**（proposal Non-Goals）：不放宽 String 实参可拼写性；不支持多个 String 实参；不处理其它描述符；不改常量体匿名子类投影判据；不泛化接口匿名路径的 `b"D"` 特化。

## 3. 验收

- [ ] 3.1 1.3 冻结的非 `op` 正例：投影成功、渲染源集 `javac --release 8` **exit 0**、`java -Xverify:all` 运行结果与原 class 一致；**`TestEnums2a/DoubleOperations`（`op` 名）呈现逐字节不变**（最重要的零回退锚）。
- [ ] 3.2 负例全部响亮拒绝；`recover-proved-string-arg-enum-constant-bodies` 的既有测试零回退；全仓测试通过。
- [ ] 3.3 门禁：`cargo test --workspace --tests --locked --no-fail-fast`（基线数字以开工时主线实测为准；已知 flake 家族见 handoff.md，含 `bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too`，单测复跑两轮判定）；`cargo fmt --all -- --check`；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成**（含 `--all-features`、29 项 `-A`、`-D warnings`）；`openspec validate --all --strict`；corpus 双腿扫描（差异应仅非 `op` 名的 String 实参枚举形；**出现其它差异类即停下报告**）；`git diff --check`。磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，**报告前必 clean**。
- [ ] 3.4 root 独立复核已证名的数据通路、四处判据替换、发射文本、`TestEnums2a` 逐字节零回退与非 `op` 正例的三方行为，更新 **DT-12** 账本（"含匿名常量体的枚举"——本缺陷所在单元；DT-10 是"空枚举、普通枚举常量"，不涉常量体，故不属本片账本范围）。（留 root）
