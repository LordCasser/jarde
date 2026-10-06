# 任务 3.3 证据：门禁记录（coder，2026-10-06）

工作区 = 隔离 worktree，HEAD = 父提交 `dccd21c3` + 本片改动（未推送）。所有命令在
`/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a110cf-daa9-7d12-8e4e-ee2e330ed22c` 下执行；
磁盘每轮构建前 `df -h /` 检查（最低实测 66Gi 可用，未触发 `cargo clean` 线）。

## fmt

```text
$ cargo fmt --all -- --check
fmt exit=0
```

（首跑在新增测试文件上失败，`cargo fmt --all` 后复跑干净；随后 `git diff --check` exit 0。）

## clippy（ci.yml 46-76 逐字生成）

任务书逐字命令（`grep -E "^cargo|^-A"`，**不含** `-D warnings`）：

```text
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.32s
clippy exit=0
```

CI 实际跑的形态另含 `-D warnings`（同一 sed 范围里第 76 行），一并复跑：

```text
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A|^-D" | tr '\n' ' ' > /tmp/ci-clippy-full.sh && sh /tmp/ci-clippy-full.sh
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.52s
clippy exit=0
```

两条均 exit 0、零 warning。**注**：任务书给的逐字命令因 `grep -E "^cargo|^-A"` 过滤掉了
`-D warnings`，单跑它不能把 warning 变红；上面第二条是同一 sed 范围补上该行的等价形态（CI 的真实命令），
两者都记录在此，供 root 复核口径。

## openspec strict

```text
$ openspec validate --all --strict
Totals: 303 passed, 0 failed (303 items)
```

（与 handoff 记的基线 303 一致；本片 tasks.md 增补说明未改变条目数。）

## git diff --check

```text
$ git diff --check
diff-check exit=0
```

## fixture census（`crates/jarde-reader/src/classfile.rs`）

新 fixture 入 `tests/fixtures/` 后实测（先跑出漂移值，再改钉值并复跑）：

```text
assertion `left == right` failed: fixture population changed: re-measure these counts
  left: (765, 3179, 286, 2009, 8)
 right: (749, 3139, 286, 2005, 8)
```

→ 钉值改为 `(765, 3179, 286, 2009, 8)`（+16 classes、+40 bodies、+4 branch targets——
`RG.pick` 的 `iflt`/`goto` 每腿两条；handler 与 subroutine 不变），复跑：

```text
$ cargo test -p jarde-reader --lib --locked repository_class_fixtures_validate_without_false_target_rejections
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 177 filtered out
```

## corpus fingerprint

```text
$ cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
$ cargo test --test p5_corpus_fingerprint --locked
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out
```

diff = `tests/fixtures/corpus-fingerprint.json` +100 行，恰为本片新增的 20 个非 Markdown 文件
（`tests/fixtures/recover-static-generic-field-init-text/` 的 16 个 class + 4 个 `.java` 源；
README 为 Markdown，不入清单），此前条目零改动。

## 全量测试

```text
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
tests exit=0
passed=3104 failed=0 ignored=61
```

（合并态基线为 3101/0/60，见 handoff；本片 +3 个非 ignored 测试与 +1 个 ignored replay。
首跑时唯一失败是 `p5_corpus_fingerprint::corpus_files_match_the_recorded_fingerprint`——新 fixture
尚未登记指纹，属预期；再生指纹后复跑 0 失败。**无 flake 家族命中**，故无需单测复跑判定。）

## 新增测试（本片）

```text
$ cargo test --test recover_static_generic_field_init_text --locked
test result: ok. 3 passed; 0 failed; 1 ignored
$ cargo test --test recover_static_generic_field_init_text --locked -- --ignored --nocapture
the real javac 8 leg is /Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
test result: ok. 1 passed; 0 failed; 0 ignored
```

可证伪性（红-绿）：把 `src/facade.rs` 还原到父提交后复跑非 ignored 三项 →
`the_broken_text_is_gone_on_both_legs` 与 `the_parent_commit_texts_are_the_broken_forms_this_change_removes`
FAILED（`the_refusals_and_the_instance_field_control_survive` 两侧均通过，它是零回退对照）；恢复修复后
三项全绿。

## 冻结树上的复跑（census 改动晚于首轮 clippy，故相关检查重跑）

```text
$ cargo fmt --all -- --check            → exit 0
$ sh /tmp/ci-clippy.sh                  → exit 0   （任务书逐字命令）
$ sh /tmp/ci-clippy-full.sh             → exit 0   （同 sed 范围 + -D warnings）
$ openspec validate --all --strict      → Totals: 303 passed, 0 failed (303 items)
$ git diff --check                      → exit 0
$ git status --short                    → 空（冻结树，4 个提交在分支 recover-static-generic-field-init-text）
```

收尾后 `cargo clean`（释放 23.2 GiB + 1.9 GiB，`df -h /` 回到 67Gi 可用），符合"报告前必 clean"纪律。
