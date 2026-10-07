# 2.5：`crosses a quoted fallback` 的所有权审计（closing-2x）

日期 2026-10-08。测试：`tests/preserve_local_scope_refusals.rs` 的
`the_crossing_fallbacks_belong_to_the_slices_they_refuse`。审计问题：当拒绝句说某局部“crosses a
quoted fallback”时，那个 fallback 是否**属于同一局部的依赖切片**——属于则保留完整拒绝（或修上游
区域证据），不属于则缩窄拒绝范围。**未改动任何 guard 证书判据**（guard 家族刚落地；本项只追踪
fallback 所有权）。

## 1. `p3_try_local` 的正向成员（`Held.use`，未变异）

* 现状：`try (java.io.Reader local1 = arg0) { int local2 = arg0.read(); return local2; }` 整段呈现，
  `fallbacks` 空、`content = contains_statements`。
* 判定：该成员**没有** quoted fallback，“crosses”问题不成立；资源副本的 def-use（定义 → header、
  body 读、编译器 close）全部由证书呈现。审计结论：保持。

## 2. `p3_try_local` 的变异成员（`2b c6 00 10` → `2b c7 00 10`，测试内复现）

javap（v9 `Held.use`，变异后）：

```
 0: aload_0   1: astore_1   2: aload_0   3: invokevirtual read   6: istore_2
 7: aload_1   8: ifnonnull 15  11: aload_1  12: invokevirtual close  15: iload_2  16: ireturn
17: astore_2  18: aload_1  19: ifnonnull 35  22: aload_1  23: invokevirtual close
26: goto 35   29: astore_3  30: aload_2  31: aload_3  32: addSuppressed  35: aload_2  36: athrow
```

区域记录（两腿相同）：

| region bci | blocks | code |
| --- | --- | --- |
| 0 | `[0, 17, 22, 29]` | `jre_region_exception_edge` |
| 11 | `[11, 15, 35]` | `jre_region_uncovered_blocks` |

副本 `local 1` 的读写：定义 `astore_1@1`（块 `[0,11)`）、null 测试读 `aload_1@7`（块 `[0,11)`）、
**close 读 `aload_1@11`（块 `[11,15)`，即 quoted 区）**、handler 侧 null 测试读 `aload_1@18`
（块 `[17,22)`）、close 读 `aload_1@22`（块 `[22,29)`）。

* 判定：**fallback 属于该局部的依赖切片**——close 的读落在被引注的 uncovered 块里，且定义与
  consumer 也在被引注的异常边区里。拒绝句 `local 1 crosses a quoted fallback region; …` 与
  `@bytecode 0 11 15 17 22 29 35` 正是这条切片的完整闭包（定义、handler、汇合/transfer、区域外
  consumer 全覆盖；见 `02-refusal-closure.md` 的同一断言方法）。
* 处置：**保留完整拒绝**。要缩窄就得把 close 写成语句，而 close 的可呈现性正是 `twr` 证明在变异
  后拒绝的东西（guard 家族的门槛），不属于本 change；删除守卫或降低断言是任务书禁止的。
* 上游证据无需修：区域记录已把两条原因（异常边、uncovered 块）分别具名，quote 与 origin 精确到块。

## 3. `p3_typed_catch::a_catch_type_of_zero_becomes_neither_a_catch_nor_a_finally`（`finallyIncrements`）

javap（v8 `TypedCatch.finallyIncrements`）：

```
0: iconst_0  1: istore_1  2: iload_0  3: istore_1  4: iload_1  5: iconst_1  6: iadd  7: istore_1
8: goto 18   11: astore_2  12: iload_1  13: iconst_1  14: iadd  15: istore_1  16: aload_2  17: athrow
18: iload_1  19: ireturn
```

* 现状：成员呈现（`contains_statements`），唯一引注块是 `[11]`（`jre_region_uncovered_blocks`，
  quote `11 12 13 14 15 16 17`），**没有** `crosses a quoted fallback/protected region` 句；
  `jre_region_exception_edge` 不在 `fallbacks` 里（2.15 的结论：该行覆盖的指令不会抛，边不可达）。
* 被呈现局部的指令与 origin：`int local1 = 0;`→BCI 1、`local1 = arg0;`→3、
  `local1 = local1 + 1;`→7、`return local1;`→19（源映射逐条定位，均在引注块之外）。
* 引注块 `[11..17)` 里的 slot-1 访问是 handler 副本自己的（`iload_1@12`、`istore_1@15`）；若它们
  属于被呈现局部，同一条 fallback 规则会像第 2 节那样触发整成员拒绝——该成员没有拒绝，说明它们是
  另一条局部身份（handler 副本的独立生命周期），不构成被呈现切片的越界使用。
* 判定：**fallback 不属于被呈现局部的依赖切片**；拒绝范围已是块级（“缩窄”已完成），无需放宽。
  保持现状。

## 4. 汇总

| fixture | fallback 是否在局部依赖切片上 | 处置 | 钉面 |
| --- | --- | --- | --- |
| `p3_try_local` 正向 `use` | 无 fallback | 保持 | 现有 `p3_try_local` 测试 + 本审计测试的对照断言 |
| `p3_try_local` 变异 `use` | **是**（close 读落在引注块 `[11,15)`） | 保留完整拒绝 | 本审计测试（quote/region/块内指令断言） |
| `p3_typed_catch::finallyIncrements` | **否**（handler 副本是独立局部身份） | 保持块级引注 | 本审计测试（呈现语句 origin + 引注块断言） |

任务书提到的“有/无 `Frame::arm` 继承修复时相同”属于历史证据：三项在 current main 上都不再是红测，
且本审计不需要任何恢复代码改动，也不需要触碰 guard 证书——若将来要呈现变异 `use` 的 close，
那是 guard 家族（`twr`/copy 证明）的切片，不是本 change 的声明规划层。
