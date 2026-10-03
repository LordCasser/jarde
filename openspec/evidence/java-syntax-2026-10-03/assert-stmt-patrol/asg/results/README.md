# `recover-assert-statement-sugar` 结果记录（三方对照、SHA、corpus 扫描）

## 回写输出（实现后 Jarde，release `class_source_file` / `class-source` 任务命令）

| 输出 | 文件 | SHA-256 |
| --- | --- | --- |
| A1 家族（fam.jar，Sub 嵌套折叠内） | [A1.jarde.java.txt](A1.jarde.java.txt) | 见下方清单 |
| AssertProbe（msg 副作用，正例） | [AssertProbe.jarde.java.txt](AssertProbe.jarde.java.txt) | 见下方清单 |
| AssertProbe-extra（守卫额外语句，负例） | [AssertProbe-extra.jarde.java.txt](AssertProbe-extra.jarde.java.txt) | 见下方清单 |
| AssertProbe-wrong-owner（负例） | [AssertProbe-wrong-owner.jarde.java.txt](AssertProbe-wrong-owner.jarde.java.txt) | 见下方清单 |
| MixedShapes（混合形态，负例） | [MixedShapes.jarde.java.txt](MixedShapes.jarde.java.txt) | 见下方清单 |

逐文件 SHA-256 见 [outputs-sha256.txt](outputs-sha256.txt)。

## 三方对照（原 class 两态 / 固定 JADX / Jarde 重编）

**A1 家族**：原 class 默认态 `10/25/2/-2`、`-ea` 态 `10/25/2` + `AssertionError`（exit 1）——
与 `../fixture/orig.out`、`orig-ea.out` 一致。Jarde 回写文本（剥两行 jarde 头注）经
`javac A1.java`（JDK 23）重编，`java A1` 与 `java -ea A1` 两态输出与原 class **逐字节一致**
（默认 `10/25/2/-2`；`-ea` `10/25/2` + AssertionError，栈行号不同为源行差异，stdout 与退出码一致）。
固定 JADX(dev) 对照沿用本层 `fixture` 家族在巡查时的三方记录口径（JADX 不回写 `assert`，
显式三件套呈现；本变更使 Jarde 的呈现形状与原源对齐，行为三方一致）。

**AssertProbe**：原 class 与重编两态均为默认 `0|0;0|0`、`-ea` `1|0;bad:-1|2|1`（msg 的
`detail(value)` 仅失败路径调用一次，求值序保持；重编后 `assert guard(arg0) : detail(arg0);`
由 javac 重新降低为同形状守卫）。09-23 审计的 wrong-owner 选择态
（`-ea:AssertProbe -da:java.lang.StringBuilder`）下原补丁类输出 `0|0;0|0`，Jarde 保持其
显式呈现不折叠——正是判据要求 clinit 行 class literal 证明的原因。

**负例**：extra-statement / wrong-owner / MixedShapes 三者在实现构建下与基线构建
（`git worktree` 干净 HEAD 腿）输出**逐字一致**（MixedShapes 实测 diff 为空；其余断言
三件合成物仍在、无 `assert` 语句）。

## corpus 双腿扫描（2165 个 class）

腿 1：干净 HEAD（`git worktree add /tmp/asg-base HEAD`）release 构建；腿 2：本变更构建。
同一脚本逐 class 转录并记 stdout SHA（[scan 脚本](scan.sh)）。

- **same = 2154**（逐字一致，含两个新增负例 fixture——它们按判据保持原呈现）；
- **text-diff = 11**，全部为 assert 家族类，diff 仅合成三件套 → `assert` 语句的形态变化：
  - `assert-stmt-patrol/fixture/A1.class`、`A1$Sub.class`（锚定家族）
  - `2026-09-23/assert-statement/AssertProbe.class`、`2026-09-22/assert-syntax/AssertProbe.class`
    及其 `root-after-class-25bf` 副本
  - `2026-09-24/assert-core/AssertCore.class` ×3（含 replay 副本）、`AssertVariants.class` ×2
  - `2026-09-26/Patrol2.class`（示例 diff：字段声明/守卫/clinit 三处 → `assert arg0 > 0 : "bad " + arg0;`）
- exit-diff = 0；both-fail 两侧一致。

逐 SHA 清单：[corpus-before.sha](corpus-before.sha)、[corpus-after.sha](corpus-after.sha)、
差异路径 [corpus-diff.txt](corpus-diff.txt)。

## 门禁（最终实测，收尾会话）

- `cargo test --workspace --tests --locked --no-fail-fast`：294 个 result 行全 ok，0 FAILED。
- `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`：299 个 result 行 ok，
  两个失败 `p4_plugins::the_plugin_plane…` 与 `ordinary_generic_projection::evidence_selection…`
  均为 handoff.md 记录的已知 flake 家族（后者即"ordinary_generic_projection 临时目录碰撞"），
  各自单测复跑 3/3 绿。
- `cargo fmt --all -- --check`：通过。
- clippy：`.github/workflows/ci.yml` 实有 `-A` 清单 **29 项**，
  `cargo clippy --workspace --all-targets --all-features --locked -- $(29 项)` 零警告。
- `openspec validate --all --strict`：**261 passed, 0 failed**（261 项；任务书所记 262 为过时计数）。
- 新增 2 个负例 fixture class 使 `repository_class_fixtures…` 计数与 corpus fingerprint 清单按流程
  重测更新（454/2151/251/1679/8；`regenerate_corpus_fingerprint` 重生成，diff 仅新增条目）。

## corpus 双腿复核（收尾会话重跑）

基线工作树（干净 HEAD 的独立 `git worktree`）与实现工作树各自的 release
`class_source_file` 重跑两腿：`corpus-before.sha` / `corpus-after.sha` 与本目录存档**逐字节一致**，
diff 仍为上述 11 个 assert 家族类（2154 同、11 异、0 exit-diff）。
