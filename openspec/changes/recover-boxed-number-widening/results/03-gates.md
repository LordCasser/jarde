# 任务 3 证据：门禁 + 语料 + 指纹/人口（change `recover-boxed-number-widening`）

工作树：本次实现的 worktree。门禁在**最终文件状态**上跑（`build.rs` 最后一次改动后未再改）。

## 门禁（逐字尾部）

1. `cargo fmt --all -- --check` — 无输出，**exit 0**（首次检查命中 `tests/recover_boxed_number_widening.rs`
   一处断言换行，`cargo fmt --all` 后复跑 exit 0；此后 `build.rs`/测试未再改）。

2. CI-exact clippy（`.github/workflows/ci.yml` 46–76 行逐字提取）：
   `sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh`

```text
    Checking jarde v0.1.0 (/…/subagent-01a11549…)
    Checking jarde-cli v0.1.0 (/…/subagent-01a11549…/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.37s
CLIPPY-EXIT=0
```

3. `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`（**authoritative**：退出码 +
   `grep -c "test result: ok"` + 零 `test result: FAILED` 行）

   **第一次运行**：exit **101**；`test result: ok` = **333**；`test result: FAILED` = **1** —
   `tests/ordinary_generic_projection.rs::frozen_identity_shapes_and_raw_control_project_together`
   在 `tests/ordinary_generic_projection.rs:54` 报 `Os { code: 17, kind: AlreadyExists }`：
   该文件 `compiled_bytes` 助手用 `jarde-ordinary-generic-{name}-{pid}-{nanos}` 做 `fs::create_dir(…).unwrap()`
   （纳秒命名撞车），是**既有 flake**，与本片无关（该文件不在本片 diff 内）。

   **flake 单测复跑 ×2（`--all-features`）**：

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 1.19s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.41s
```

   **第二次全量运行（干净）**：exit **0**；`test result: ok` = **334**；`test result: FAILED` = **0**；
   合计 **3159 passed / 0 failed / 77 ignored**（逐 `test result: ok` 行求和）。含本片新目标
   `tests/recover_boxed_number_widening.rs`（3 passed + 1 ignored）；基线 333 目标 → 334（+1 = 本片目标）。

4. 语料移动纪律：`cargo test --test p3_execution_comparison --all-features --locked -- --ignored`

```text
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.75s

ORACLE-EXIT=0
```

   无需更新任何陈旧 oracle 期望（语料移动未触及 oracle 断言面）。

5. `openspec validate --all --strict`：

```text
✓ change/type-immediate-functional-receivers
Totals: 307 passed, 0 failed (307 items)
```

   **exit 0**。

6. `git diff --check` — 无输出，**exit 0**。

7. 磁盘：建前/建后 `df -h /` 均 >15Gi（本片无 `cargo clean` 触发）；最终报告前按纪律清理。

## 语料（两条差分，均带自测与自头断言）

1. **候选面**（目标描述符 `Ljava/lang/Number;`；散装 16 + jar 条目 6 = **22**；类名由 `javap` 声明行解析，
   负 fixture 的文件名≠内部名也已覆盖，非渲染 0）——`results/corpus-sweep.out`：

```text
SELF-TEST OK: known positives recover; known negative BNX 2 -> 2 refusals
loose candidate classes (target descriptor in the pool): 16
archive candidate classes: 6
MOVED: openspec/evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/fixture/C8.class (refusals 1 -> 0)
MOVED: openspec/evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/fixture/c8.jar!C8.class (refusals 1 -> 0)
moved classes: 2
non-render candidates (identical on both binaries, not counted): 0
refusal sentences across the rendered candidates: 2 -> 0
```

2. **全量语料**（`openspec/evidence/**` 全部 **1987** 个 class，不问描述符；基线 vs patched 双腿渲染逐字节
   diff）——`results/full-corpus-diff.out`：

```text
self-test: …/boxed-number-widening-patrol/fixture/C8.class moved=yes (want yes)
self-test: …/boxed-number-widening-patrol/fixture/C7.class moved=no (want no)
MOVED: openspec/evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/fixture/C8.class (refusals 1 -> 0)
classes compared: 1987
non-render classes (identical on both binaries, not counted): 0
moved classes: 1
```

**增量分类**：移动 = 巡查锚 `C8` 自身（候选面两处渲染：散装 class 与 `c8.jar` 条目；全量面一处文件），
引注 **2 → 0**、新增引注 **0**、非渲染 **0**；其余 1986 个 class 逐字节不动。无额外收益、无回退、无新拒绝。

## 指纹与 fixture 人口（按先例再生）

- `tests/fixtures/corpus-fingerprint.json`：`cargo test --test p5_corpus_fingerprint --locked -- --ignored
  regenerate_corpus_fingerprint` → diff = **+45 行 / 0 删除**（9 个条目：3 源 `.java` + 6 class），其余条目
  逐字节不变；复跑校验测试 5 passed / 0 failed。
- `crates/jarde-reader/src/classfile.rs` 的 fixture 人口：`(857, 3723, 290, 2273, 8)` →
  **`(863, 3757, 290, 2309, 8)`** = +6 类 / +34 body（17/腿）/ +0 handler / +36 branch target（18/腿）/
  +0 subroutine；注释按先例追加一段写明本片三类的构成与分支来源，实测复跑该测试 exit 0。
