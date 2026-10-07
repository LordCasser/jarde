# 3.1 全门禁（逐字 tail）

所有命令在本 worktree（`/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11750-…`）执行，
最终编辑（fmt）之后重跑。

## fmt

```
$ cargo fmt --all -- --check
（无输出）
$ cargo fmt --all && cargo fmt --all -- --check
FMT CLEAN
```

## clippy（ci.yml 46–76 的逐字命令）

```
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
    Checking jarde-query v0.1.0 (…/crates/jarde-query)
    Checking jarde-jvm v0.1.0 (…/crates/jarde-jvm)
    Checking jarde-java v0.1.0 (…/crates/jarde-java)
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 47.07s
```

（无 warning / error；`-D warnings` 未触发。）

## workspace 测试（权威口径 = 退出码 + ok 行数 + FAILED 行数）

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast > /tmp/ws-final.log 2>&1; echo "exit=$?"
exit=0
$ grep -c "test result: ok" /tmp/ws-final.log
340
$ grep -c "test result: FAILED" /tmp/ws-final.log
0
$ tail -3 /tmp/ws-final.log
test local_variable_target_preserves_its_table_and_charges_each_entry ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

无已知 flake 家族被触发（本片未触碰 bulk 并发/取消族；未出现 FAILED 行，故无需单测重跑 ×2）。

## openspec

```
$ openspec validate --all --strict
✓ spec/structural-xref
✓ change/type-immediate-functional-receivers
Totals: 312 passed, 0 failed (312 items)
```

## oracle 腿（强制）

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.50s
```

## 本片自己的双腿回放（ignored）

```
$ cargo test -p jarde --test recover_io_resource_finally -- --ignored
test the_anchors_own_class_answers_its_baseline_and_its_boundary_is_stated ... ok
test the_mid_read_guard_answers_what_its_class_answers ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s
```

## 语料指纹 / 普查 / 计费

见 [05-corpus-and-oracle.md](05-corpus-and-oracle.md)（指纹 +27 文件纯增；普查 +20 fixtures/+46
bodies/+24 handler records/+22 branch targets；计费表 `analysis_steps` +3 三处，附归因；oracle 腿无
过期期望）。
