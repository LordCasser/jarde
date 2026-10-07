## 1. 基线与负例

- [x] 1.1 重放固定 C7/C8（SHA 核对）：反射核对直接边全集（design 取证义务）；记录 BCI 63 基线。
      → 实测：C7/C8 三条 SHA 与巡查 `fixture-sha256.txt` **逐字相同**；基线二进制渲染 `c8.jar` 在 BCI 63 得
      **同句逐字**拒绝（引注 1）。反射核对（`results/probe/number-universe.out`，rt.jar sha256 `b27515a6…` 一致）：
      java.lang 直接子类**恰六**、间接 **0**，表外 5 直接 + 4 间接如实记录，`Number` 自身
      `superclass=Object`/`interfaces=[Serializable]`。
      **前提漂移（如实记录）**：(a) 呈现形态由巡查时的"部分体 + 丢行"变为今天的"整成员引注"（期间呈现规则变更，
      拒绝句/BCI 不变）；(b) `String→CharSequence` 与八装箱→`Comparable`/`Serializable` 已由 2026-10-06 的
      姊妹片落地——本片剩余 delta 恰为六条 `→ java.lang.Number` 边。
- [x] 1.2 冻结至少三个变体/负例：Double/Long 装箱变体、String→CharSequence、Boolean→Number（拒绝）；各自 `java -Xverify:all` 前后记录。
      → 实测：`tests/fixtures/recover-boxed-number-widening/`（巡查 `C8.java` 逐字节复制 + 本片 `BN`/`BNX`，
      两条 javac 腿，命令与 SHA 见 fixture README）。Double/Long 变体与六行全覆盖：base 8 引注 → patched 0；
      `String→CharSequence` 变体（`BN.pickSeq`）**base 即 0 引注**（姊妹片已覆盖，本片不动其呈现）；
      `Boolean→Number` 在 javac 源级**不可产生**，钉在 `build.rs` 单元测试；可产生的表外真实子类负例
      （`BNX`：`BigDecimal`/`AtomicInteger`）2 引注前后**逐字不变**。`java -Xverify:all` 前后行为（两腿相同）：
      C8 `x/1:2/7/eoeoeoe`、BN `7/7/7.5/7.5/7/7/yy/9/4`、BNX `2/2`（SHA 见 `results/02-rows-anchors-tests.md`）。

## 2. 闭集扩展

- [x] 2.1 java.lang 边入表+walk；C8 恢复、重编行为逐字一致（`x`/`1:2`/`7`/`eoeoeoe`）；既有闭集与用户类负例 diff 零回退。
      → 落表 `NUMBER_FAMILY`（六行，`platform_interface_argument_widens` 同函数、同 release-8 门、同
      `cast_argument` 呈现）。**walk 按实测收窄（如实记录）**：反射核对证明 java.lang 直接边全集=六、无中间节点，
      且该函数任一表的目标都不是另一张表的源（闭包=行集），walk 会是死代码，故不写。C8 恢复：
      `results/cli-roundtrip.out` 双腿编译 + `-Xverify:all` 运行与原 class 逐字一致；语料**全量 1987 个 class**
      双腿渲染 diff **仅 C8 一类移动**（引注 1→0、新增引注 0），零回退（`results/full-corpus-diff.out`）。
- [x] 2.2 预算/取消不变。
      → diff 不触任何预算/取消代码路径；`cargo test --locked --test generic_method_budget`（`<T extends Number>` 形参族）
      4 passed / 0 failed；工作区全量见 `results/03-gates.md`。

## 3. 回归与验收

- [x] 3.1 全仓测试全绿（含 throwable/collection widening 全部既有测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
      → 逐字尾部见 `results/03-gates.md`；已知 flake 家族单测复跑 ×2 判定（temp-dir 命名冲突，见该文件）。
- [x] 3.2 C8 与变体三方对照；记录输出 SHA。
      → 三方（源 class / 装机 javac 腿 / 真 javac 8 腿）对照与渲染/输出 SHA 见 `results/02-rows-anchors-tests.md`
      与 `results/cli-roundtrip.out`。
- [ ] 3.3 root 复核闭集逐对与三方行为，更新 EM 账本与巡查记录。（留 root）
