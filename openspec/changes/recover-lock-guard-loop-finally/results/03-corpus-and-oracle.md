# Corpus 差分与 oracle 腿

## 1. fixture 指纹（`corpus-fingerprint.py`，列 = 输入 sha256 / 渲染 sha256 / 路径）

`fingerprint.txt`：`tests/fixtures/**/*.class` 全部 867 个提交件逐一渲染。
与上一片（`preserve-local-scope-across-exception-regions/results/planning-1.2-1.3/fingerprint.txt`，
本切片的父提交）逐行比较：

```
common files: 867  new: 0  moved: 0
```

零移动。（本切片新增的 6 个 fixture class 会作为 new 行出现在 `fingerprint.txt` 里；它们是本切片自己的输入。）

## 2. 全证据语料差分（`02-corpus-diff.sh`，`02-corpus-diff.out`）

`openspec/evidence/**` 下**全部 1987 个 class 文件**，分别用基线二进制（父提交 `6f8ed47a`，独立 worktree +
独立 target）与本切片二进制渲染，逐字节比较：

```
corpus classes: 1987
baseline renders done
patched renders done
self-test: the anchor moved (want moved)
self-test: the healthy control did not move (want no)
classes compared: 1987
non-render classes (identical on both binaries, not counted): 0
moved classes: 0
```

**零移动**：本切片在 1987 个证据类上不放宽、不收紧任何判定。自测：锚（`lk.jar`）必须移动、健康对照
（`tests/fixtures/p3-handlers/v8/Guarded.class`）必须不动——两者都按预期。

## 3. oracle 腿（corpus-moving discipline）

```
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
running 3 tests
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.25s
```

**无陈旧期望**：本切片没有移动 oracle 语料的任何样本（与 §1/§2 的零移动一致），所以没有需要更新的
断言——不是"跳过"，而是实测后确认无需更新。

## 4. 三份差分如何互补

| 面 | 覆盖 | 结果 |
| --- | --- | --- |
| fixture 指纹（867） | 全部提交的测试夹具 | 零移动 |
| 全证据语料差分（1987） | 全部提交的证据类（含所有巡查夹具与负例） | 零移动 |
| oracle 腿（3 测试，编译+执行） | 行为级重放（javac 编译 + `-Xverify:all` 运行） | 3/3 通过，无期望更新 |
| 本切片测试（5 测试） | 锚三法恢复 + 双腿重编重跑 + 负例逐字 | 5/5 通过 |
