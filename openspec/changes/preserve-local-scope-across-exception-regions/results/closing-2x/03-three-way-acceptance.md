# 2.3：固定 SHA 的三方行为验收（closing-2x）

日期 2026-10-08。CLI = 本 worktree 的 `target/debug/jarde-cli`。脚本
`03-three-way.sh`（自测：每个渲染先断言自头部、真 javac 8 路径存在、被拒形状在两条腿上都编不过），
输出 `03-three-way.out`；测试 `tests/preserve_local_scope_refusals.rs` 的
`the_legal_shapes_answer_what_the_original_and_jadx_answer`（`#[ignore]`，需两条 JDK 腿）。

## 1. 固定输入

| 输入 | 角色 | SHA-256 |
| --- | --- | --- |
| `preserve-local-scope-plan/v8/ScopePlan.class` | 原类（javac 23.0.1 `--release 8 -g:none`） | `c3f37e50298002a6af481479be56a07a888a16e942097fc0396df2206119c0d7` |
| `preserve-local-scope-plan/v8-javac8/ScopePlan.class` | 原类（真 javac 8） | `ca467a7fc66aa9d5cb9339626f36b33dff8f402fbdc2c93d2709cd5050b480e3` |
| `preserve-local-scope-plan/jadx/ScopePlan.java` | JADX 完整类（固定 checkout `2fb1b1638694` 渲染） | `24bcbce9d890e27bb8df6c5ac9251c3ba6a490ce0822f98d6f4e46500cd2c4b4` |
| `preserve-local-scope-plan/ScopePlanDriver.java` | 行为驱动（正常 + 异常输入 + 逃逸 Error） | `ae27f334e30de02d42fbef4b98c52e30f1fb69781e63bf696a471871ff373fa0` |
| `p3-loop-try-handler-entry/v8/LoopTryHandlerEntryArgs.class` | 1.5 形原类（javac 23.0.1） | `e83048988366411b83e814ed69a3e3c22d8461c264bf7debfdfde14dd17ad6a5` |
| `p3-loop-try-handler-entry/LoopTryHandlerEntryArgs.jadx.java.txt` | 1.5 形 JADX 完整类 | `1cc23ff89b197ef5ed75a1dcec9e987bc2ada3c903d00e3854292352af31ec39` |
| `p3-loop-try-handler-entry/Runner.java` | 1.5 形驱动（normal + caught） | `01a68ce896d9cf14d78021c4edb5407a84f08886d9df762168e29a2c3cb94d6d` |

JADX 渲染由固定 checkout 生成：

```sh
/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
    -d /tmp/jadx-scopeplan --no-res --no-imports \
    tests/fixtures/preserve-local-scope-plan/v8/ScopePlan.class
# → sources/defpackage/ScopePlan.java（本切片按原样提交，含 JADX 的 package 行）
```

## 2. 合法形状：原类 / JADX 完整类 / Jarde 完整类三方对照

两条编译器腿（安装版 `javac --release 8` 与真 javac 8 Corretto 1.8.0_432）各自编译三方源码，
以 `java -Xverify:all` 执行，逐行比对（`03-three-way.out` 全文）：

| shape | leg | original | jadx | jarde |
| --- | --- | --- | --- | --- |
| `ScopePlan` | installed | 10 行 trace | 同 | 同 |
| `ScopePlan` | real | 10 行 trace | 同 | 同 |
| `LoopTryHandlerEntryArgs` | installed | `normal=6` / `caught=0` | 同 | 同 |
| `LoopTryHandlerEntryArgs` | real | `normal=6` / `caught=0` | 同 | 同 |

`ScopePlan` 的 trace 覆盖每个成员的两条输入路径与一个**逃逸异常**：

```
catchOnly false=1 / catchOnly true=2
assignedAcrossTry false=3 / assignedAcrossTry true=4
assignedAcrossIf false=6 / assignedAcrossIf true=5
nestedHandlerOnly null=9
nestedAcross false=10 / nestedAcross true=11
nestedHandlerOnly error=java.lang.AssertionError:boom
```

（`true` 输入使受保护区抛 `NullPointerException` 并被 handler 接住；最后一行由驱动传入一个抛
`AssertionError` 的 `Runnable`，两条子句都不接，异常类型与消息逐字比对。）

`LoopTryHandlerEntryArgs` 的 `caught` 输入使循环内 try 的 `maybeFail` 抛出并被 catch 接住
（1.5 的计算写入形）。

## 3. 拒绝形状：只验收完整 bytecode/origin 覆盖与 refusal 标记

`ScopeRefusals` 的三个被拒成员（见 `02-refusal-closure.md`）在两条腿上都**编不过**——脚本与测试
各自断言 `javac` 非零退出，并写明这不算语义通过；它们的验收是 quote/origin 覆盖与拒绝句本身。

## 4. 结果

* 测试：`cargo test --test preserve_local_scope_refusals --all-features --locked -- --ignored`
  → `1 passed`（两条腿、两个合法形状、被拒形状的不可编译断言全过）。
* 脚本：`sh 03-three-way.sh` → 表格如上，自测全过。
* 结论：合法形状的返回值、trace、异常类型三方逐项一致；拒绝形状不被当作已生成的 Java 计数。
