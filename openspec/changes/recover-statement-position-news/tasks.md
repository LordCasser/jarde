## 1. 基线与负例

- [x] 1.1 重放固定 B5/B6（SHA 核对）：读语句位判定处与实参分类数据面；记录 B6 四形基线。（`results/01-gating.md`：三件 fixture 与 `results/fixture-sha256.txt` 逐字节一致；拒绝点为 `verify` 的 `written.is_empty()` 分支——`renders_its_reads` 不含 `pop`；数据面为 `operands`/`operations`/`ssa`/`nested_sites`；门控实验=仅放行 reader 即翻转 B5/B6 记录而 CST 不翻转，且暴露"站点被接受但语句未写"的静默丢失面）
- [x] 1.2 构造并冻结至少三个变体/负例：多语句混合、静态嵌套类语句 new、CST 冻结反例（逐字不变断言）；各自 `java -Xverify:all` 通过并记录实现前后行为。（`tests/fixtures/recover-statement-position-news/`：`SP.mixed`（多语句混合 + 静态嵌套 `new Inner()`）、`SPN` 四负例、`SPC.class`（手工汇编的语句位 CST 孪生，原类 `-Xverify:all` 输出 `CST`）；两腿 `javac --release 8`/真 javac 8；实现前后行为见 `results/04-class-level.out`、`results/04-three-way.out`）

## 2. 语句位呈现

- [x] 2.1 判据呈现（design 决策 1）；B5.main 与 B6 的 argless/withArg 恢复、整类重编运行与基线逐字一致；CST 反例不变（**含拒绝码不变**——反例拒绝应仍由 `Invoke ∉ argument_dependencies` 分支产生，先于语句位 reader 检查，用测试钉死）；`chained`（实参含 getfield）保持拒绝并登记为遗留边界。（`init.rs` 的 `discarded` 判据 + `build.rs` 的 `discarded_construction`；`tests/recover_statement_position_news.rs` 的两个反例测试钉死 `jre_new_interleaved_effect` @BCI 4 与 reader 检查的先后；`chained` 仍在 `jre_new_shape` 拒绝；`results/07-acceptance-news.txt` 为终树记录）
- [x] 2.2 消费位构造与既有 new@1 通道 diff 零回退；预算/取消不变。（`results/05-corpus-delta.md`：消费位控制逐字不动、既有通道自拼写有消费位对照实证；`results/02-implementation.md` 记预算/取消无新增；全仓测试含预算与取消套件）

## 3. 回归与验收

- [x] 3.1 全仓测试全绿（含 refuse-unconsumed-construction-invokes 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。（`results/06-gates.md` 逐字记录：`331` ok / `0` FAILED / EXIT=0、`p3_ordinary_new_invokes` 2/2、clippy 零警告（含 `-D warnings`）、`307 passed, 0 failed`、`DIFF-CHECK-OK`、FMT-OK；首轮四处陈旧预期各自更新或再生，见同文件表）
- [x] 3.2 B5/B6 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（`results/04-three-way.sh/.out`：B5 三腿同为 `667c0fe7…`、SP 三腿同为 `724195f8…`、SB 三腿同为 `2e6d31a5…`（与 B6 原类同答 `9`）；B6 与 `SPC` 的重编文本保持拒绝（安全形），`SPC` 原类输出 `CST`）
- [x] 3.3 root 独立复核判据边界、CST 保护与三方行为，更新账本与巡查记录。（root 2026-10-07 完成，见 [verification-root.md](verification-root.md)：门控复核 ✓（含实现者自查抓回 owns 重写回退的证据）、CST 顺序双测试钉死 ✓、B5 剥离运行与 orig.out 逐字一致 root 亲测 ✓、门禁 331 targets ok/0 FAILED + fmt 合并态 + oracle 3/3 ✓、D3 池形与两处他片期望更新裁定追认 ✓；账本与巡查记录随本验收关闭）
