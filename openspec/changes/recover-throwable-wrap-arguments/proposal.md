## Why

[CF-15 扩验巡查](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/README.md)发现异常包装重抛家族输出不可编译：`catch (E e) { throw new RuntimeException("w:" + e.getMessage(), e); }` 中构造调用的参数 1 需要 `E → java.lang.Throwable` 引用上转型，`build.rs` 转换分派（约 20445–20496）的五个回答（Object 目标/同名/数组闭集/`reference_overload_calls` 证明/`platform_reference_argument_widens` 单对 List→Iterable）都不命中，构造拒绝后局部声明与 throw 级联被引。该家族（`new RuntimeException(msg, e)`、`new ServletException(e)`、用户 `log(Throwable)` 收异常实参）是反编译高频形态。

## What Changes

- 在调用实参转换分派中新增 java.lang 异常类闭集上转型回答：被呈现类型为 java.lang 核心 Throwable 子类（RuntimeException、IllegalStateException、IllegalArgumentException、IOException 家族、Exception、Error 等闭集）且要求类型为其 java.lang 祖先（Throwable/Exception/RuntimeException）时，按既有 widening 模式保留要求类型拼写（`cast_argument`），不引入运行时风险（上转型恒成功）。
- 闭集以固定表维护（与 `platform_reference_argument_widens` 同风格）；用户自定义类的层级证明登记为升级路径（触发条件：首个非 java.lang 类上转型场景出现时以 resolution 环境事实建 `reference_overload_calls` 同款证明通道）。
- 以 C2.alias 固定类、包装重抛变体族（不同异常类型/嵌套包装/用户 Throwable 形参方法）、既有转换回归（Object 目标/数组/overload 证明/List→Iterable）与 verifier 有效负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：java.lang 异常子类到其祖先的调用实参上转型可呈现，包装重抛与异常转发调用完整恢复。

## Impact

仅 `crates/jarde-java` 私有 `build.rs` 转换分派及测试；不新增公开 IR/CLI/依赖。非 java.lang 类的上转型继续拒绝（负例钉死）；返回位置与非调用上下文不在本片。
