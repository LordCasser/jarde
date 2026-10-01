## Why

[捕获 ctor 巡查](../../evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/README.md)确认：javac 8 对捕获型匿名/局部类（`val$x` 合成字段）与非静态成员内部类（`this$0`）的构造器发出"**合成字段 putfield 先于 super 调用**"的字节码（JVM 合法）；Jarde ctor 呈现按字节码序逐字输出 `this.val$base = arg1; super();`——Java 源码要求 super() 为首句，输出**不可编译**（javac 报"灵活构造器是预览功能"）。该模式覆盖所有捕获型/内部类构造器，高频。

## What Changes

- ctor 呈现层识别"前缀合成字段存 + super"序列：store 字段为合成（名字匹配 `this$N` 模式或 `val$` 前缀，且字段本身被呈现为合成来源字段）且存值为参数直传（aload_n 直传，无计算），整组移至 super 调用之后按原序呈现；super 成为 ctor 首句。
- 合成字段声明保留呈现（faithful，不隐藏）；非合成 pre-super 字段写（人为字节码）不重排——保持现有呈现/拒绝。
- C1（两个匿名捕获类）与 C2（this$0）family 联编 `javac --release 8` 通过且行为一致；既有全部 ctor 呈现（枚举、普通类、enum 匿名子类委托）逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：javac 合成构造器的 pre-super 字段存以 super-先行序呈现，输出合法 Java。

## Impact

仅 `crates/jarde-java` 私有 ctor 呈现（语句序规范化）及测试；无新证明机制（判据为字节码模式 + 合成字段事实）。既有 ctor 切片零回退。
