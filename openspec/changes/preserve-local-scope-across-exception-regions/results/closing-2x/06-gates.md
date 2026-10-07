# closing-2x 门禁（最终文件状态）

全部命令在最终文件状态上执行。测试新增 1 个 target（`tests/preserve_local_scope_refusals.rs`）。

## 1. fmt

```
$ cargo fmt --all -- --check
（无输出，exit 0）
```

## 2. clippy（ci.yml 46-76 逐字）

```
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
    Checking jarde v0.1.0
    Checking jarde-cli v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.49s
（0 warning，exit 0）
```

（首轮曾报 1 条 `clippy::single_match`，位于新测试文件的 OutputBytes 停止判断；已改为 `if let`
并重跑，见上。）

## 3. 全工作区测试（权威口径）

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
test result: ok 行数 = 343
test result: FAILED 行数 = 0
累计 passed = 3197（ignored = 90）
```

（对前一片的 342 targets：本片新增 `preserve_local_scope_refusals` 一个 target，其余不变。）

## 4. openspec

```
$ openspec validate --all --strict
Totals: 314 passed, 0 failed (314 items)
```

## 5. P3 执行对照 oracle（强制；语料移动纪律）

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.69s
EXIT=0
```

无需更新任何 oracle 期望：本片不改恢复代码（`crates/` 仅 census 计数更新），语料渲染不可能移动。

## 6. 本片新增/更新的定向测试

```
$ cargo test --test preserve_local_scope_refusals --all-features --locked
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out

$ cargo test --test preserve_local_scope_refusals --all-features --locked -- --ignored
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test --test preserve_local_scope_plan --all-features --locked
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 7. 语料/census 控制

* `cargo test -p jarde-reader --lib repository_class_fixtures_validate_without_false_target_rejections --locked`
  → ok（计数 945→950 / 4057→4077 / 400→418 / 2519→2540，见 `07-corpus-and-oracle.md`）。
* `cargo test --test p5_corpus_fingerprint --locked` → ok（重生成后 diff 恰为本片 9 个新文件）。
