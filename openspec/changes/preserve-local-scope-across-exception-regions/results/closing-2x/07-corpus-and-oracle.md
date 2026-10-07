# 语料 delta 与 oracle（closing-2x）

## 1. 恢复代码改动：无

`git diff --stat` 对 `crates/jarde-java`、`src/` 为空；对 `crates/jarde-reader` 只有 fixture census
的计数与 ledger 段落（`crates/jarde-reader/src/classfile.rs`）。因此本片不可能移动任何恢复渲染；
oracle（`p3_execution_comparison --ignored` 3/3）无需更新期望。

## 2. fixture census（`classfile.rs::repository_class_fixtures_validate_without_false_target_rejections`）

| 量 | 旧 | 新 | delta |
| --- | --- | --- | --- |
| classes | 945 | 950 | +5 |
| bodies | 4057 | 4077 | +20 |
| handler records | 400 | 418 | +18 |
| branch/switch targets | 2519 | 2540 | +21 |
| subroutines | 8 | 8 | 0 |

逐项归因（`tests/fixtures/preserve-local-scope-refusals/`）：

* **+5 classes**：`v8/ScopeRefusals`、`v8/ScopeRefusalsEscape`、`v8-javac8/ScopeRefusals`、
  `v8-javac8/ScopeRefusalsEscape`、派生控制 `escaped/ScopeRefusalsEscape`。
* **+20 bodies**：`ScopeRefusals` 每腿 7（`<init>`、`savedAcrossFinally`、`handlerComputed`、
  `nestedHandler`、`siblingKept`、`quotedSliceKept`、`log`）×2 = 14；逃逸类 2（`<init>`、
  `sharedHandler`）×3 = 6。
* **+18 handler records**：`ScopeRefusals` 每腿 6（`savedAcrossFinally` 1、`handlerComputed` 1、
  `nestedHandler` 2、`siblingKept` 1、`quotedSliceKept` 1）×2 = 12；逃逸类 2 行 ×3 = 6。
* **+21 branch targets**：`ScopeRefusals` 每腿 9（`savedAcrossFinally` 的 `goto`、
  `handlerComputed` 的 `goto`/`ifnonnull`/`goto`、`nestedHandler` 两个 `goto`、`siblingKept` 的
  `goto`、`quotedSliceKept` 的 `ifle`/`goto`）×2 = 18；逃逸类 `goto` ×2 = 2；派生控制 1。
* 无 subroutine（无 `jsr`/`ret`）。

`ScopePlan` fixture 的扩展（`jadx/ScopePlan.java`、`ScopePlanDriver.java`）不新增 `.class`，
故 census 不变。

## 3. corpus fingerprint（`tests/fixtures/corpus-fingerprint.json`）

```
$ cargo test --test p5_corpus_fingerprint --locked            # 先失败：9 个 unlisted
$ cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
wrote tests/fixtures/corpus-fingerprint.json
$ git diff --stat tests/fixtures/corpus-fingerprint.json
 tests/fixtures/corpus-fingerprint.json | 45 ++++++++++++++++++++++++++++++++++
```

diff 恰为 9 个新文件（每个 5 行：path/bytes/blake3 与括号），无其它行变动：

```
tests/fixtures/preserve-local-scope-plan/ScopePlanDriver.java
tests/fixtures/preserve-local-scope-plan/jadx/ScopePlan.java
tests/fixtures/preserve-local-scope-refusals/ScopeRefusals.java
tests/fixtures/preserve-local-scope-refusals/ScopeRefusalsEscape.java
tests/fixtures/preserve-local-scope-refusals/escaped/ScopeRefusalsEscape.class
tests/fixtures/preserve-local-scope-refusals/v8-javac8/ScopeRefusals.class
tests/fixtures/preserve-local-scope-refusals/v8-javac8/ScopeRefusalsEscape.class
tests/fixtures/preserve-local-scope-refusals/v8/ScopeRefusals.class
tests/fixtures/preserve-local-scope-refusals/v8/ScopeRefusalsEscape.class
```

（`.md`/`.py` 按 manifest 的既有排除规则不入指纹；`patch-escape.py` 与 README 因此不计。）

## 4. oracle

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.69s
```

无 stale 期望；无断言删除或降低。
