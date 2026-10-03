## 1. 取证与冻结边界

- [x] 1.1 核对 `recover-nested-generic-class-headers` 的实际验收状态，重放 Z1 的 SHA、8 个方法和 4 个字段诊断；逐项定位 `project_method_signature`、字段投影、类级成员遍历及来源交接，形成使用点与拒绝原因表。验证：表中每项能对应物理成员、原始 `Signature`、descriptor 和产生诊断的代码路径；未验收的嵌套头不计为本项成功。
- [x] 1.2 冻结最小顶层 Java 8 正反类：无竞争的同类方法调用、同类字段读写、未被消费的 CP 引用、相邻重载改绑、字段遮蔽、未完整解码或未呈现使用点。验证：每类保存源、class SHA、`javap -v -c`、`java -Xverify:all` 原始行为和当前 Jarde 输出；变造 class 先过 JVM 校验。

## 2. 使用点证明与原子投影

- [x] 2.1 在现有类源码准备流程内列举有界的同类实际成员使用点，核对调用者、BCI、opcode、owner/name/descriptor 与完整扫描状态；CP 中未被任何已支持来源消费的条目不算使用。验证：1.2 的未使用 CP 正例可区分于直接调用、字段操作和 bootstrap/未知来源负例；预算/取消及部分解码不会得到“无使用”结论。
- [x] 2.2 对无竞争的同类方法调用证明源码接收者、实参静态类型与原物理目标一致，沿用现有 Signature/擦除/正文判据发布泛型方法候选。验证：最小类完整源码 `javac --release 8`、独立调用方和泛型反射与原 class 一致，调用 BCI 可追溯；无关方法文本不变。
- [x] 2.3 对相邻同名重载逐调用位核对可见候选、适用性和最具体选择；证据不足或绑定会改变时保持原拒绝。验证：1.2 的可证/不可证变体分别通过目标 descriptor 与运行对照、拒绝码/来源断言，不能因输出能编译而放行错误目标。
- [x] 2.4 对同类字段读写证明源级字段绑定、赋值类型及受泛型字段类型影响的消费者，再发布可证的字段 Signature 候选。验证：字段正例的完整类重编、字段泛型反射、读写值及效果顺序相同；遮蔽、未知消费者和类型冲突负例保留 `field_generic_body_unproved` 或更精确拒绝及物理 BCI。
- [x] 2.5 对互相影响的方法/字段候选先完成闭合证明、来源和输出计费，再整体提交已证组；失败不污染无关成员。验证：组合正反类、低预算、预取消/中途取消、essential/all 和输出上限均无半个泛型头或伪完整报告，独立方法报告与原始成员身份不变。

## 3. 行为与门禁

- [x] 3.1 对 1.2 的成功子集保存原 class、固定 JADX Java-input 和 Jarde 三方源码与哈希；Jarde 完整类和独立调用方以 `javac --release 8` 重编，`java -Xverify:all` 比较调用目标、字段值、异常及泛型反射。验证：逐路径结果与原 class 相同；JADX 的偏离只作为对照，不作为准入依据。
- [x] 3.2 重放 Z1 家族，逐项标出本变更使 `generic_call_binding_unproved`、`field_generic_body_unproved` 消失或保留的证据，并复跑相邻泛型、重载、字段、类级装配与预算回归。验证：未证明位仍明确拒绝，嵌套头、lambda、成员类的独立缺口不计入本项通过。
- [x] 3.3 执行适用的全工作区测试、`cargo fmt --all -- --check`、仓库 CI 的严格 Clippy、`openspec validate --all --strict --no-interactive`、语料 census/fingerprint 和 `git diff --check`；记录实际通过项与既存失败，不机械改门禁数字。验证：附命令、输出摘要和磁盘占用；Cargo 编译临时 target 在验证后清理。
- [x] 3.4 对源码绑定证明、真实消费者闭包、原子提交和三方行为作独立复核，记录通过与剩余拒绝边界。验证：复核者按 1.2 固定输入重放至少一个正例、一个重载负例、一个字段负例及一个预算停止，并核对物理来源。

### Root 复核记录（合并主线 2a69683e）

- 3.3：root 实测——全仓 `--tests` 2898/0；fmt 通过；CI 实有 29 项 `-A` clippy 干净；`openspec validate --all --strict` 258/258（一次瞬时竞态重跑排除）；`p5_corpus_fingerprint` 5/5；`git diff --check` 干净；target 验证后清理（47Gi）。
- 3.4：root 重放——正例 `add(T)`/`Map<String, List<T>> index` 投影（marker 含 same-class uses BCI）；重载负例（同 arity 兄弟）、接收者位字段负例、预算停止原子性、Z1 逐项共 9/9 绿。`items`/`first`/`map` 的保守拒绝为 design 边界正确执行（投影将发布不可编译源）。实现者自纠的 opcode 误写（corpus 双腿捕获）为门禁体系有效性实例。剩余边界（同 arity 泛型兄弟/函数位/接收者位）登记下一前沿。
