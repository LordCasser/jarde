# Baseline freeze source-hash erratum v1

本勘误修正 `baseline-freeze.md` 与 `baseline-freeze.json` v1 的 direct fixture source hash 身份错误。v1 把 candidate 清单 `reports[*].source_sha256` 误标为原始输入源码 hash；这些值实际对应 Jarde 生成的 Java 源码。错误不影响 class/JAR、CLI/product、JADX archive/hash、BCI/反汇编、raw stream 或结构拒绝结论。

v1 历史文件保留为 `baseline-freeze-v1.md`（SHA-256 `fb2cbb4aab603095425f0da73b08277181fa4cde0ad8a6eb9a95ddb6e6df20a0`）和 `baseline-freeze-v1.json`（SHA-256 `48d1aa14760854f32c53274a4902358a0cfc28f3da1f6664eb346a7526c36058`）。root 对 v1 的审计记录为 `baseline-freeze-root-verification.json`，SHA-256 `1fe2bb5a5a5a7043097f0262787b965e9ba1b0624f4c38ac4412fa75133623f4`，33 checks / 6 issues；该结果保留为历史，不覆盖。

原始输入源码以 canonical fixture 路径 `openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/direct/` 为准；`v3-initial/direct/` 有相同内容与 hash。原始输入 hash 为：

| 源文件 | 原始输入 SHA-256 |
|---|---|
| `Base.java` | `d32b225a5e129346a2bef89edfc9ab7388336af1d65c1c9f6a4286572c4d2df2` |
| `DerivedA.java` | `d33f1ed9b05f59e5d5e077c34c2e056df286e2b63962ddac5c51be13ed9f9977` |
| `DerivedB.java` | `62ea0c550b93e2e50cfa97e079ef7036445c669929d6e0b485aa6bceb0b4c47c` |
| `LocalInterface.java` | `425f56e2492e46e3f32206b6e338f5faebd5fadb6dba2391a4048164f73cc621` |
| `Mid.java` | `c6f88e2194880bedd5fff9bf2ef1757af27b7dd6b48efc239cf3ed16cfc1b770` |
| `Main.java` | `e4e9189086f9417c523c4db0cadce69739c66f9e6c297ccc4a7914768f29a017` |

v1 曾列出的另一组 hash 并未作废，正确身份是 Jarde 生成源码：这些值来自 `results/candidate-v1-fixture-v3/manifest.json` 的 direct legs 中 `reports[*].source_sha256`。对应生成文件位于 `results/candidate-v1-fixture-v3/fixture/direct/javac8/sources/` 与 `javac23/sources/`。v2 JSON 将它们单独记录为 `jarde_generated_source_sha256`；当前报告明确区分 generated source 与原始输入，不改前片 manifest、输入 fixture 或历史 root 记录。
