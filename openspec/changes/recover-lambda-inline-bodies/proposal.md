## Why

[lambda 巡查](../../evidence/java-syntax-2026-10-03/lambda-inline-patrol/README.md)实锤：lambda 恢复通道健康（零引用、语法呈现），但 lambda 表达式与伴生 `lambda$…` 合成方法同时呈现——javac 对 lambda 表达式重合成同名方法，**命名空间冲突使任何含 lambda 的类整类不可编**（Y1 固定复现）。Java 8 核心形态，必须消除。

## What Changes

- 伴生 `lambda$…` 方法体内联进 lambda 表达式（MVP：单表达式/直线体——形参按位绑定到 lambda 参数、无控制流语义差），伴生方法在呈现中隐藏；伴生仅被该 lambda 引用（单用途证明）。
- 复杂体（多语句/分支/return 语义差）保守：伴生保留但重命名为非冲突名（`lambda$…$jarde` 后缀；private static 合成语义不变），整类可编。
- Y1 整类重编 `javac --release 8` 通过、运行与 orig.out 逐字一致（`hi!`/`45`/`[b, aa]`/`8`）；无 lambda 类与既有 functional-receiver 通道零回退；`::` 方法引用语法不在本片（行为已对，登记后续）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：lambda 表达式呈现伴生体内联（直线体）或非冲突伴生名（复杂体），含 lambda 的类可重编。

## Impact

呈现/装配层（伴生方法与 lambda 位点的配对、体内联与形参绑定）及测试；invokedynamic 证明通道不动。既有 functional-receiver/lambda 相关切片零回退。
