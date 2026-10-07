# 1.2/1.3 切片：门禁与语料证据

全部命令在最终文件状态上执行；`git diff` 对 `crates/`（除下方 census 两处）与 `src/` 为空——
本切片不改变恢复行为，语料 delta 因此可归零解释。

## 1. fmt

```
$ cargo fmt --all -- --check
（无输出，exit 0）
```

## 2. clippy（ci.yml 46-76 逐字）

```
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.47s
（0 warning，exit 0）
```

## 3. 全工作区测试（权威口径）

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
test result: ok 行数 = 335
test result: FAILED 行数 = 0
累计 passed = 3164
```

（同命令的前一次运行出现一次 `p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched`
失败，唯一差异是 `elapsed_millis` 0 vs 1：属已知计时抖动族；该单测 `--all-features` 重跑 ×2 均绿，
随后整轮重跑 EXIT=0。见 `/tmp/workspace-tests2.log` 与 `/tmp/workspace-tests3.log`。）

## 4. openspec

```
$ openspec validate --all --strict
Totals: 307 passed, 0 failed (307 items)
```

## 4b. P3 执行对照 oracle（强制）

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.13s
EXIT=0
```

无需更新任何 oracle 期望（本切片不移动语料渲染）。

## 5. 语料 delta

* 恢复代码零改动：`git diff --stat crates/jarde-java src` 为空 → 渲染行为不可能移动。
* 两处 **census 断言更新**（新增 fixture 的必然结果，非行为改动）：
  * `crates/jarde-reader/src/classfile.rs` 的 fixture census 元组
    `(863, 3757, 290, 2309, 8)` → `(867, 3777, 312, 2333, 8)`，并按其既有 ledger 体例补一条
    本 fixture 的条目（4 类 = 2 类 × 2 leg；20 body = 每 leg 10；22 handler 记录 = 每 leg 11；
    24 分支目标 = 每 leg 12；子程序仍 8）。
  * `tests/fixtures/corpus-fingerprint.json` 由该测试自己的 regenerator 重生成，diff 恰为本 fixture
    的 6 个新文件（2 源 + 4 class），无其它行变动。
* 站立零回归渲染控制（`baseline/`，clean HEAD 二进制）：`LK`、`IO`、`ExceptionScope`、`ScopePlan`、
  `ScopePlanCrossing` 与 `gating/a`、`gating/b` 的对照见 `02-gating.md`。
* 本切片的语料指纹：`fingerprint.txt`（867 个 `tests/fixtures/**/*.class`，逐文件 class 摘要 + 渲染摘要；
  33 个条目带 `:exit4`，即 CLI 的 withheld-projection 非零退出但仍产出渲染）。

## 6. 行为对照

`04-roundtrip.sh`（自测通过，日志 `roundtrip.log`）：正例类文本剥注释后由安装版 `javac --release 8`
与真 javac 8 重编，两个重编类在 `java -Xverify:all` 下与 fixture 自身 class 逐行同答
（`1,2,3,4,6,5,9,10,11`）。
