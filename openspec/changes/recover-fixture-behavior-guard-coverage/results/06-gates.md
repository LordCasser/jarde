# 3.1 门禁实录（实现者，2026-10-07）

工作树：`/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a114b3-51b1-7e73-a02d-ef993197db80`
（隔离 worktree，detached HEAD = `fdef2537`）。工具链：`java`/`javac` 23.0.1。日志文件都在本目录。

## fmt

```text
$ cargo fmt --all -- --check
FMT OK        (exit 0)
```

`cargo fmt --all` 已对新测试文件与改动文件执行过；`--check` 复跑无差异。

## clippy（从 `.github/workflows/ci.yml` 46–76 行逐字生成）

生成方式（逐字，`results/` 里 `04-gates-clippy.out` 记录了生成出的命令与 29 项 `-A` 计数）：

```text
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
$ grep -o -- '-A ' /tmp/ci-clippy.sh | wc -l
29
```

- **逐字形**：`sh /tmp/ci-clippy.sh` → **exit 0**（`04-gates-clippy.out`）。
  注意该 grep 模式**不含** `-D warnings`（`-D` 不匹配 `^cargo|^-A`），而 ci.yml 第 76 行正是
  `-D warnings`——故另跑一次把该行补回的 **CI 等价形**。
- **CI 等价形**：`sh /tmp/ci-clippy-deny.sh`（逐字形 + `-D warnings`）→ **exit 0**（`04-gates-clippy-deny.out`）。

两者都 `Finished dev profile … in ~15s`、无 warning。

## workspace 全目标测试

命令（CI 的两趟 `cargo test --workspace --all-targets --all-features --locked` 加 `--no-fail-fast`，
父任务书指定）：

```text
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
```

AUTHORITATIVE 判定 = 退出码 + `grep -c "test result: ok"` + `test result: FAILED` 行数（不做宽口径 awk）。
本机（macOS，桌面负载 load average ≈ 7–9）连跑数轮，逐轮的判定与 flake 处置（前四轮的失败块原文收在
`06-flake-runs-extract.out`，逐例的判定复跑原文收在 `06-flake-adjudication-*.out`）：

| 轮次 | 退出码 | `test result: ok` | `test result: FAILED` | 失败测试 | 处置 |
| --- | --- | --- | --- | --- | --- |
| 1 | 101 | 331 | 1 | `p3_two_exit_return::complete_class_compiles_and_matches_eight_verified_paths` | scratch 目录 `AlreadyExists`（同族：handoff「scratch 目录 AlreadyExists」家族）；单测复跑两轮 7 passed/0 failed（`06-flake-adjudication-p3_two_exit_return.out`） |
| 2 | 101 | 330 | 2 | `d3_artifact_binding::the_evidence_is_rebuilt_after_every_temporary_of_the_first_request_is_dropped`（`elapsed_millis` 0 vs 1）、`engine::standalone_engine_reports_owned_source_and_usage` | 两者都是 handoff 已登记 flake 家族（`d3_artifact_binding` 见 handoff 2026-10-05/10-06 两条、`engine::standalone` 见家族清单）；单测各复跑两轮绿（`06-flake-adjudication-run2.out`） |
| 3 | 101 | 331 | 1 | `p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched` | handoff 家族清单明写 `p4_plugins`（计时）；差为 `elapsed_millis` 0 vs 1；单测复跑两轮绿（`06-flake-adjudication-run3.out`） |
| 4 | 101 | 331 | 1 | 同上（`p4_plugins` 计时） | 同族、同断言；单测复跑两轮绿 |
| **5（`06-workspace-tests.out`）** | **0** | **332** | **0** | — | **权威轮：干净**（`grep -c 'Running tests/'` = 319 个测试二进制） |

每轮都包含新守卫二进制 `tests/fixture_behavior_guards.rs`（全量跑只执行默认套件）：
`test result: ok. 7 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out`。

**flake 判定依据**（按 handoff 的纪律）：失败测试都不在本片改动面（本片零生产代码、只加测试与文档），
每例失败都是计时/临时目录碰撞断言（`elapsed_millis` 差 1、`AlreadyExists`），且每例单测复跑两轮全绿；
干净轮（第 5 轮）在同一棵树上 332 ok / 0 FAILED / exit 0。

```text
$ tail -1 06-workspace-tests.out
workspace test exit=0
$ grep -c 'test result: ok' 06-workspace-tests.out
332
$ grep -c 'test result: FAILED' 06-workspace-tests.out
0
$ grep -c 'Running tests/' 06-workspace-tests.out
319
$ sed -n '598p' 06-workspace-tests.out
test result: ok. 7 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

基线对照：tasks.md 记的开工基线是 root 2026-10-04 环 1 合入后的 296 目标 / 2943 passed；本片新增
1 个测试二进制（7 默认 + 6 ignored），本轮 319 个测试二进制 / 332 条 `test result: ok`。

## `#[ignore]` 腿单独复跑

```text
$ cargo test --test fixture_behavior_guards --locked
test result: ok. 7 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.03s

$ cargo test --test fixture_behavior_guards --locked -- --ignored
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
```

（两轮都在扰动回退之后复跑过：见 `README.md` §2.3。）

## `openspec validate --all --strict`

```text
$ openspec validate --all --strict
…
Totals: 307 passed, 0 failed (307 items)      (exit 0)
```

## corpus fingerprint（零 corpus 位移）

```text
$ cargo test --test p5_corpus_fingerprint --locked
test corpus_files_match_the_recorded_fingerprint ... ok
test the_manifest_matches_its_rendered_classification ... ok
test every_dimension_is_carried_by_existing_corpus ... ok
test every_acceptance_row_is_indexed_against_existing_corpus ... ok
test the_indexed_rows_are_the_rows_the_acceptance_table_names ... ok
test result: ok. 5 passed; 0 failed; 1 ignored
```

`tests/fixtures/corpus-fingerprint.json` **未改动**（`git status` 无该文件）：本片新增的 fixture 侧文件
全是 `.md`（README），而指纹把 `md` 作为"关于语料的记录"排除在输入之外——故本片没有移动任何 corpus
字节，也没有触碰任何 census/fingerprint 文件。

## `git diff --check`

```text
$ git diff --check            (工作区已暂存全部改动后)
(no output)  exit 0
```

## 3.2 零生产代码改动

```text
$ git diff --stat HEAD -- crates src
(empty)
```

生产面（`crates/`、`src/`）零改动；扰动自检的那一处生产改动已完整回退（`README.md` §2.3）。
