## Why

普通 `(String)x`、`((String)x).length()` 和数组引用转换已具备完整 `checkcast` 事实，但恢复层只接受 bridge 规则认领的擦除，其余均退回字节码引用。自写 CastProbe 的八个方法及同 class 的 javac、jadx、jarde 对照已保存在 `openspec/evidence/java-syntax-2026-09-22/casts/`。

## What Changes

- 已知目标引用类型与操作数的普通运行时检查呈现为显式转换，支持返回、局部、调用消费及数组目标。
- 保持嵌套检查顺序、单次求值、null 与 ClassCastException 行为，以及转换和生产者的来源。
- 结果被丢弃或后续消费拒绝时保留检查和延期生产者的引用，不输出非法的裸转换语句，也不把检查视为无副作用值。
- 保留已证明的 bridge 呈现路径；用自写 fixture 的真实恢复文本重编译对照，而非以 jadx 为执行预言机。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加显式引用转换及其失败保留契约。

## Impact

前置条件为现有 SSA、CheckCast 事实、Cast AST、类型名拼写、消费位置检查和来源引用。主要修改 `jarde-java` 的值构造及直接消费/失败保留分支，配套独立 fixture/test；无需新增 AST、pass、crate、类型解析器或生产依赖。

非目标：数值转换、instanceof、任意类层次/泛型推断、catch/循环等区域准入扩展、重排或消除检查、伪造可编译性或验证状态。八方法中的 catches 另受保护块准入限制，不以本项完成宣称整个方法恢复。
