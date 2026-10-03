## Why

[装箱 Number 巡查](../../evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/README.md)确认：`<T extends Number>` 泛型方法以装箱值调用（`larger(3, 7)`——Integer 实参→擦除 Number 形参）无上转型证据 → 语句被引、重编丢行。`platform_reference_argument_widens` 闭集缺 java.lang 装箱家族边（六数值装箱→Number、全部→Comparable、String→CharSequence/Comparable）。

## What Changes

- java.lang 闭集直接边入表 + 传递闭包 walk（同 throwable/collection 先例、JDK 8 反射机械核对）；C8.main 恢复且行为逐字一致。
- 既有两闭集（throwable 45 对、collection 40 对）与 `List→Iterable`、用户类负例零回退；`cast_argument` 保留要求类型拼写。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：java.lang 装箱/字符串家族的调用实参上转型可呈现。

## Impact

仅 `crates/jarde-java` 白名单函数及测试；复用闭集先例，无新机制。
