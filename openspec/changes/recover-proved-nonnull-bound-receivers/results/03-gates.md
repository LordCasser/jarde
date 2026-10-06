# 任务 3.1 门禁记录（coder，2026-10-06；逐字命令与实测尾部）

代码状态：`5549a7e0`（feat）+ `9d572ee6`（test）+ `3e05dae5`（corpus 重测）+ 本提交（docs）。
全部在隔离 worktree 内运行，**未 push**。

## 1. `cargo fmt --all -- --check`

```text
$ cargo fmt --all -- --check
（无输出，exit 0）
```

## 2. clippy（从 `.github/workflows/ci.yml` 46–76 逐字生成）

命令生成与执行（**29 项 `-A`**，`--all-features` 在内）：

```sh
sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
sh /tmp/ci-clippy.sh
```

生成出的命令原文存 [`03-clippy-command.sh`](03-clippy-command.sh)（单行，29 个 `-A` 项）。
实测尾部（[`03-clippy-tail.txt`](03-clippy-tail.txt)）：

```text
    Checking jarde v0.1.0 (...)
    Checking jarde-cli v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.83s
```

`exit 0`，**0 warning / 0 error**（`grep -c "^warning\|^error"` = 0）。

## 3. `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`

```text
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
exit 0
TOTALS passed=3097 failed=0 ignored=59
targets with a `test result:` line: 320
```

逐 target 的 `Running … / test result:` 行存 [`03-tests-per-target.txt`](03-tests-per-target.txt)（640 行）。
与父提交 `dba93835` 的验收基线（3094/0/58）相比：**+3 passed、+1 ignored**，全部来自本片
（`tests/recover_proved_nonnull_bound_receivers.rs` 2 passed + 1 ignored，`init::tests` 新增 1 passed）。

**已知 flake 家族的实测分类（`export_cli` 计时）**：同一代码（最终树 `3d3734d8`）的三次整仓运行中，
一次在 `-p jarde-cli --test export_cli::a_counted_dimension_override_is_accepted_and_stops_the_run_at_that_dimension`
失败（`left: Null` vs `right: "cli_export_unfinished"`——并行负载下该 CLI 子进程运行的计时断言），另两次
（3097/0/59）全绿；按纪律单测复跑两轮均绿：

```text
$ cargo test -p jarde-cli --test export_cli --locked a_counted_dimension_override   # ×2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in ~1.25s
```

判定：**已知 `export_cli` 计时家族**（handoff 清单内），非本片回归——依据是同代码异结果（另两次整仓绿，
含父提交基线同代码的绿跑）＋单测两轮绿。未复现第二次。

## 4. `openspec validate --all --strict`

```text
$ openspec validate --all --strict
Totals: 302 passed, 0 failed (302 items)
```

## 5. 本片自身的定向复跑（最终提交树上）

```text
$ cargo test --test recover_proved_nonnull_bound_receivers --locked
test result: ok. 2 passed; 0 failed; 1 ignored

$ cargo test --test recover_proved_nonnull_bound_receivers --locked -- --ignored
test every_stripped_anchor_answers_what_its_class_answers ... ok
test result: ok. 1 passed; 0 failed

$ cargo test -p jarde-java --lib --locked bound_receiver
test init::tests::the_bound_receiver_tail_is_claimed_exactly_where_the_receiver_is_proved ... ok
```

## 6. 其它实测（非门禁命令）

- corpus 渲染差分：`02-corpus-sweep.sh`（自检先行）→ moved = 5（全部解锁形，1 → 0 拒绝），
  unrendered = 13（全部为有意损坏/字节补丁负例与 `package-info`），见 [`02-corpus-sweep.out`](02-corpus-sweep.out)。
- corpus 指纹再生：`cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`
  → `tests/fixtures/corpus-fingerprint.json` 仅 +6 条（2 源 + 4 class），零删除。
- reader 普查：`cargo test -p jarde-reader --lib --locked` → 178 passed（计数更新为
  `(739, 3083, 284, 1833, 8)`）。
