# 任务 3.1 门禁记录（coder，2026-10-06，worktree `subagent-01a11096`，HEAD 5a9f32bf + 本片改动）

全部命令在 worktree 根目录执行，逐字如下；尾部输出逐字贴出。磁盘纪律：每轮构建前 `df -h /`（本轮
最低 48Gi 可用，未触发 `cargo clean`）。

## `cargo fmt --all -- --check`

```text
（无输出）
FMT_EXIT=0
```

## clippy（ci.yml 逐字）

命令（`sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' '`，
存档 [`03-clippy-command.sh`](03-clippy-command.sh)）：

```text
cargo clippy --workspace --all-targets --all-features --locked -- -A clippy::too_many_arguments -A clippy::cloned_ref_to_slice_refs -A clippy::collapsible_if -A clippy::type_complexity -A clippy::len_zero -A clippy::needless_option_as_deref -A clippy::needless_borrow -A clippy::useless_conversion -A clippy::large_enum_variant -A clippy::question_mark -A clippy::comparison_to_empty -A clippy::op_ref -A clippy::manual_range_patterns -A clippy::if_same_then_else -A clippy::filter_map_bool_then -A clippy::filter_next -A clippy::unneeded_struct_pattern -A clippy::redundant_guards -A clippy::map_identity -A clippy::redundant_slicing -A clippy::unnecessary_get_then_check -A clippy::unnecessary_unwrap -A clippy::redundant_locals -A clippy::replace_box -A clippy::map_clone -A clippy::unnecessary_mut_passed -A clippy::single_element_loop -A clippy::unnecessary_to_owned -A clippy::needless_lifetimes
```

尾部（完整输出 [`03-clippy-tail.txt`](03-clippy-tail.txt)；`warning` 计数 0）：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
CLIPPY_EXIT=0
```

## `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`

```text
（321 个 target 全部 `test result: ok`；逐 target 存档 03-tests-per-target.txt）
TOTALS passed=3101 failed=0 ignored=60

CARGO_EXIT=0
```

本片新增的 target `tests/recover_loop_else_if_early_returns.rs`：4 passed / 0 failed / 1 ignored
（ignored 为需 JDK 的剥离回放，单独实跑见下）。已知 flake 家族（`ordinary_generic_projection`、
`bulk_recovery_*`、`export_cli`、`p4_plugins`、`engine::standalone`）本轮全部一次通过，无需单测复跑。

## 剥离回放（ignored 单跑，两腿两 javac）

```text
cargo test --test recover_loop_else_if_early_returns --locked -- --ignored --nocapture
the real javac 8 leg is /Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 5.33s
```

## `openspec validate --all --strict`

```text
（尾部）
✓ change/type-immediate-functional-receivers
Totals: 303 passed, 0 failed (303 items)
```

## 语料指纹与 reader 普查（`5d0be186`/`80d500f5` 先例）

```text
cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.20s
cargo test --test p5_corpus_fingerprint --locked
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

`tests/fixtures/corpus-fingerprint.json` 的差分为**纯新增 15 条**（5 个源 + 10 个 class，两腿），无任何
既有条目改动（`git diff` 实测：`+76/-1`，其中 `-1` 是 diff 头）。

reader 普查（`crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections`）
由 `(739, 3083, 284, 1833, 8)` 移到 `(749, 3139, 286, 2005, 8)`：+10 类（5 类 × 2 腿）、+56 body
（28/腿：`BS` 6、`CB` 6、`CB2` 4、`LB` 8、`LR2` 4）、+2 handler（`LB.tryLadder` 的 catch，1/腿）、
+172 分支目标、subroutine 不变；注释按先例补写。
