## Why

[InnerClasses family 巡查](../../evidence/java-syntax-2026-10-01/inner-enum-args-patrol/README.md)确认 family 末片：`TestEnumsInterface` 形态（enum implements I，常量经**匿名子类**构造）折叠失败退化为逐字段呈现。字节码结构已预研：`<clinit>` 每常量 `new Sub; dup; ldc "NAME"; iconst ord; invokespecial Sub.<init>(String,int); putstatic NAME`；匿名子类 `final class Sub extends <enum>` 的 ctor 体恰为委托 super 调用（枚举自身 ctor 带合成 `$1` 参防递归），覆盖方法体在 Sub 类内。架构事实：`ClassSourceNestedEnumFamily::Prepared` 已携带子类完整报告（含逐方法恢复文本）——本片是既有装配通道上的扩展，无需新机制。

## What Changes

- 枚举常量折叠接受"常量经匿名子类构造"形态：常量步骤的 `new` 目标为经 member/nested-family 通道准备的**子类报告**，且该子类满足——InnerClasses 关系为该枚举的成员、`extends` 本枚举、ctor `(String,int)` 体恰为 `aload_0; aload_1; iload_2; aconst_null; invokespecial <enum>.<init>(String,int,合成$1参)` 委托、无其它成员除覆盖方法外（覆盖方法的 descriptor 与本枚举或其接口的实例方法匹配）。
- 呈现：该常量折叠为 `NAME { <子类覆盖方法恢复文本，含 @Override 标注> }`；子类不再作为独立嵌套类出现在输出中。
- MVP 边界：仅覆盖方法恢复为 structured 的子类参与折叠；任一子类体方法拒绝/停止 → 整枚举保持逐字段呈现（保守）；无匿名体常量与四固定形/任意实参路径零变化。
- N2 全量折叠（`PLUS { @Override public int apply… }`、`MINUS { … }`、enum 头含 `implements IOperation`）；整 family（N2+IOperation+匿名类消除）三方重编运行一致。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：匿名子类构造的枚举常量折叠为常量专属体，子类呈现并入常量花括号。

## Impact

根 crate `src/enum_constants.rs`（常量步骤接受 Sub new + 子类义务证明）与 `src/class_source.rs`（匿名子类的家族准备与呈现合并、子类独立呈现抑制）及测试。复用 nested/member-family 通道与子类报告的既有恢复；无新跨类机制、无公开 API 变化。既有 enum 全部证书零回退。
