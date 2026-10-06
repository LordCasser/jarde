# 任务 3 证据：门禁 + 语料指纹/人口 + 增量分类（change `recover-temporal-argument-widening`）

工作树：本次实现的 worktree（`subagent-01a10fc3…`），门禁在**同一最终文件状态**上跑（feat 提交
`a53814a1` 之后；此后只有 test/证据文件新增，未再改 `build.rs`）。

## 门禁（逐字尾部）

1. `cargo fmt --all -- --check` — 无输出，**exit 0**。
2. CI-exact clippy（`.github/workflows/ci.yml` 46–76 行逐字提取，另补该块末行 `-D warnings`）：
   `sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh`
   （+ `-D warnings`），`sh /tmp/ci-clippy.sh`：

```text
    Checking jarde v0.1.0 (/…/subagent-01a10fc3-f5f3-7cc2-9605-d104ce4ef086)
    Checking jarde-cli v0.1.0 (/…/subagent-01a10fc3-f5f3-7cc2-9605-d104ce4ef086/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.76s
```

   **exit 0**（`-D warnings` 在场，0 warning）。

3. `cargo test --workspace --all-targets --all-features --locked`（完整日志 `/tmp/tw-render/workspace-tests.log`）：

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

WORKSPACE-TESTS-EXIT=0
```

   聚合：**318** 个测试目标（`running` 行数），`passed = 3085`、`failed = 0`、`ignored = 58`
   （逐 `test result:` 行求和；无 `FAILED`/`panicked` 行）。含本片新目标
   `tests/recover_temporal_argument_widening.rs`（3 passed + 1 ignored）。

4. `openspec validate --all --strict`：

```text
✓ change/type-immediate-functional-receivers
Totals: 301 passed, 0 failed (301 items)
```

   **exit 0**。

5. 本片锚的 ignored replay（需要 JDK，单列）：

```text
the real javac 8 leg is /Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
test every_stripped_anchor_answers_what_its_class_answers ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 1 filtered out; finished in 4.26s
```

## 语料人口/指纹（按 `cfd54879`/`6f1dd48e` 先例再生）

- `tests/fixtures/corpus-fingerprint.json`：`cargo test --test p5_corpus_fingerprint --locked --
  --ignored regenerate_corpus_fingerprint`。diff = **+12 个条目**（4 个源 `.java` + 8 个 class），
  其余条目逐字节不变（`git diff --stat` = 60 插入 / 0 删除）。
- `crates/jarde-reader/src/classfile.rs` 的 fixture 人口（`repository_class_fixtures_…`）：
  `(691, 2943, 282, 1827, 8)` → **`(699, 2989, 282, 1827, 8)`**，即 **+8 类、+46 body**，
  handler/branch/`jsr` 计数不变（本片四个 fixture 的两条腿各 23 个 Code body：`JT` 5、`DT` 4、
  `CF` 11、`TWX` 3）。注释行按先例追加一段写明来源；实测复跑该测试 exit 0。

## 增量分类（"只有本片行移动的人口/拒绝"）

| 类别 | 实测 | 分类 |
| --- | --- | --- |
| 渲染面 | 语料（1987 散装 class + 全部 jar 条目）里目标描述符候选 **3** 个，恰为本片三锚；引注 **6 → 0**，新增引注 **0** | 全部是本片行放行的拒绝；无额外收益、无回退 |
| 散装语料 | 候选 **0**（1987 个 class 里没有任何 `java.time`/`CompletableFuture` 调用点） | 不动 |
| 既有测试 pin | 工作区 3085 项全绿，**未改任何既有测试** | 0 处移动（与 charsequence 片不同：本片没有既存 pin 指向 `Temporal`/`CompletionStage` 对） |
| fixture 人口/指纹 | 见上（+8 类/+46 body；+12 指纹条目） | 书账，随新 fixture |
| CF 注记 | `combined()` 恢复后新增两条 `// jarde: omitted physical lambda helper …` 注记与两条内联注记 | 恢复的伴随（fold 对已证明成员写注记），非渲染回退 |

完整输出：`results/corpus-sweep.out`（含自测行 `SELF-TEST OK: known positive JT 4 -> 0 refusals;
known negative TWX 2 -> 2`）、`results/corpus-anchor-diffs.txt`。
