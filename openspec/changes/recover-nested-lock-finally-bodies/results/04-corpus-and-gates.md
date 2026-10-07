# 语料普查、指纹与门禁

## 语料普查（baseline vs current，945 个 class 全渲染）

[corpus-fingerprint.py](corpus-fingerprint.py)（沿用 LK 片的普查脚本，CLI 由 `JARDE_CLI` 指定）
对 `tests/fixtures/**/*.class` 的每一个 class 渲染一次并配对输入/渲染摘要：

```
945 class file(s) fingerprinted into …/fingerprint-baseline.txt     (二进制 = 3e45800a)
945 class file(s) fingerprinted into …/fingerprint-current.txt      (二进制 = 本片实现)
```

逐行比对（同输入、渲染摘要不同者）：

| 路径 | base | current |
| --- | --- | --- |
| `tests/fixtures/recover-nested-lock-finally-bodies/v8/ML.class` | `7349a5d5…` | `3ae307c1…` |
| `…/v8-javac8/ML.class` | `7349a5d5…` | `3ae307c1…` |
| `…/v8/MLOrder.class` | `7496e75b…` | `186be69b…` |
| `…/v8-javac8/MLOrder.class` | `7496e75b…` | `186be69b…` |

**4 处移动，全部是本片两个锚类**（两条重编腿各一）；其余 941 个 class 的渲染逐字节不变。
（`fingerprint-baseline.txt` 与 `fingerprint-current.txt` 存本目录。）

对**上一份**指纹（LK 片 `results/fingerprint.txt`，873 行，早于其后数片落地）的分类：

- 共同路径渲染移动 4 处，均为**中间片**的既有 delta（`ScopePlanCrossing` 两腿 = LK 片自己
  更新的负例、`LockGuardProbe` 两腿 = IO 片的行集准入）；本片 baseline→current 对它们**零移动**
  （上表即证）。
- 新增路径 72 个 = 中间片（IO、instance-chains、covariant、short-circuit、double-brace、
  proved-java-structure）的 60 个 + 本片 12 个；删除 0 个。

## 提交语料指纹（`tests/fixtures/corpus-fingerprint.json`）

`cargo test --test p5_corpus_fingerprint --locked` 在实现前点名 18 个未登记文件（本片 6 源 + 12
class）；`-- --ignored regenerate_corpus_fingerprint` 重生成后：

```
tests/fixtures/corpus-fingerprint.json | 90 ++++++++++++++++++++++++++++++++++
1 file changed, 90 insertions(+)
```

**纯增**：18 条新条目（6 源 + 12 class），既有条目的 blake3/字节数一行未动；重生成后
`cargo test --test p5_corpus_fingerprint --locked` → `5 passed; 0 failed; 1 ignored`。

本片随后加入第三条登记边界（`MLProbe.nestedLocksBranching`：分支体使释放副本自成一块、canonical
把方法自身的 `return` 融进该块）后，清单再记一次 **3 条重测**（`MLProbe.java` 源 + 两条腿的
`MLProbe.class`，706→945 字节），其余条目仍一行未动；重测后同命令绿。

## 读者侧语料人口断言（corpus-moving 纪律）

`crates/jarde-reader/src/classfile.rs::repository_class_fixtures_validate_without_false_target_rejections`
在实现后按设计红：`fixture population changed: re-measure these counts`，
`left = (945, 4057, 400, 2519, 8)` vs `right = (933, 3993, 368, 2475, 8)`。按既有惯例**更新断言**
（不删断言）：期望值改为测量值，并补一段本片 fixture 的登记注释（12 class、64 body、32 handler
record、44 branch target、0 subroutine；每腿 32 body / 16 row / 22 branch）。更新后该测试绿。

## 门禁（本工作树，最终态）

```
$ cargo fmt --all -- --check                      # 见下方逐字尾（clean）
$ sh /tmp/ci-clippy-strict.sh                     # ci.yml 46-76 的同一命令 + `-D warnings`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.39s
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
    WORKSPACE-EXIT=0
    grep -c "test result: ok"      = 342
    grep -c "test result: FAILED"  = 0
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
    ORACLE-EXIT=0
$ openspec validate --all --strict
    Totals: 314 passed, 0 failed (314 items)
```

（342 = 上一片 341 个 target + 本片新增测试文件 `tests/recover_nested_lock_finally_bodies.rs`；
314 = 上一片 313 + 本 change 自身。）首次全量跑在读者人口断言处红一次（上节），更新后同命令全绿，
无其它红——无已知 flake 家族出现，故无单测重跑。

### 逐字门禁尾（最终态，权威）

```
$ cargo fmt --all -- --check
fmt-exit=0

$ sh /tmp/ci-clippy-strict.sh        # = ci.yml 46-76 的命令 + 该 job 尾部的 `-D warnings`
    Checking jarde-cli v0.1.0 (…/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.39s

$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
    test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    WORKSPACE-EXIT=0
  # 权威合计：exit 0 + `grep -c "test result: ok"` = 342 + `grep -c "test result: FAILED"` = 0

$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.86s
    ORACLE-EXIT=0

$ openspec validate --all --strict
    Totals: 314 passed, 0 failed (314 items)

$ cargo test --test recover_nested_lock_finally_bodies --locked -- --ignored
    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.41s
$ cargo test --test recover_io_resource_finally --locked -- --ignored
    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.40s
```

`/tmp` 占用（清理前）：`/tmp/gate`（4 个门控二进制 63M×4 + 渲染 456K + 输出 776K）、
`/tmp/mlwork`、`/tmp/rt`、`/tmp/guard-impl.rs`（参考副本）、`/tmp/ci-clippy*.sh`；均为本片
临时件，收尾清除（`/tmp/scriptos-*` 为其它会话的，不动）。

