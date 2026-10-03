## 1. 基线与负例

- [x] 1.1 重放固定 Y1（SHA 核对通过，`Y1.class` `fdc6a6de…`、`Y1$StrFn.class` `e7bab1f1…`）：读单静态行选择点（`scan_family_root` 尾部 `static_members.pop() → Candidate`）与改道面、`Candidate` 通道其它消费方隔离（`prepare_physical_class_source` 仅对静态行证 target；枚举/注解投影对走各自 scan）；巡逻基线 `a7762a92` 独立重放 + corpus 双案预扫（`sif/results/corpus-*.txt`）。**取证结论：变更前提不成立**，判别量是根文本的 lambda 投影而非子行种类（`sif/README.md` §1–§3）。
- [x] 1.2 构造并冻结 16 个变体/负例（`sif/fixtures/`，含 Y1 形态去 lambda 的 `Y1X` 反例、只换子行种类的 `Y1M`、接口专属锚缺口的 `WCallI`/`WCallC` 对、抽象方法类 `V2`、非静态 `V4`、注解 `V5`、多子 `V6`），全部 `java -Xverify:all` 通过并记录实现前后行为（`sif/README.md` §6）。

## 2. 通道统一

- [ ] 2.1 **未实施——取证判定两案均不成立，Y1 未折叠，故不勾选**（`sif/README.md` §3）：案 A 改道使 `VN3` 报告面/退出码变化（新增 `anonymous_child_shape_unproved`、exit 4）且不折叠 Y1；案 B 无准入可加（行判据自 `75a0114c` 已含 0x0608），真正接口专属判据是 `prove_static_family_target` 的 `()V` 构造器要求，补后仍不折叠 Y1。Y1 的实际阻塞为折叠根重投影门 + token 锚定门（两门对 `class` 子同样生效）。既有家族 M1/M2/FV* 逐字不变（受控核对）。
- [x] 2.2 负例边界正确（16 变体判定与 `4da1fe84` staging 片一致，缺口非本批引入）；预算/取消未触及（本片无产品码改动）。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（HEAD 干净树，1 ignored）；`cargo fmt --all -- --check` 通过；CI `.github/workflows/ci.yml` 完整 29 项 `-A` 清单 clippy `-D warnings` 通过；`openspec validate --all --strict` 258/258 通过；磁盘纪律遵守（每轮前后 `df -h /`、探针 target 用后即清）。**注**：以上验证跑在无产品码改动的 HEAD 上，只证明本片未引入回归，不构成 2.1 的实现验收。
- [ ] 3.2 **不适用/未完成**：本片无实现交付，没有可对照的 Y1 折叠产物。已记录 Y1 物理输出（`sif/fold-outputs/Y1-HEAD.jarde.java`）与既有家族折叠文本 SHA（§1/§6）；另对接口锚补丁的 `F1` 折叠产物做了重编核对（`javac --release 8` 0 错误、`java -Xverify:all` 与原 class 逐字一致），但该补丁不是本变更的实现。
- [ ] 3.3 root 独立复核通道选择、corpus 等价性与三方行为，更新账本与巡查记录。

## 结论

本变更的验收（Y1 折叠）**未达成**；取证证明其两案均不成立，`sif/README.md` 记录否证与真实边界。
建议：改写/撤回本 change，另立触及共享契约的切片（折叠重投影已投影的根文本 + 健全的类型 token
锚），并把接口 `InterfaceMethodRef` 锚补丁作为其中独立一小步。详见 `sif/README.md` §3–§5、§8。