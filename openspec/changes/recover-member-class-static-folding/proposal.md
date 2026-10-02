## Why

[成员类折叠巡查](../../evidence/java-syntax-2026-10-03/member-class-folding-patrol/README.md)确认大颗粒缺口：直接静态成员类/接口不折叠进外围文本——多子被 one-child 扫描显式拒绝（M1 五子）、单子通过扫描但消费面是匿名/枚举投影对而非声明折叠（M2 无 `static class Solo` 声明）；外围信封/体内对成员类的引用全部池 `$` 直拼（nn 片登记的类头/字段/throws 遗留位同源）。枚举折叠（常量+匿名子类+名字重写投影）已建全部先例机械。

## What Changes

- 直接静态成员类/接口（任意数量）折叠进外围类文本：按外围 InnerClasses 直接行的源序嵌入各子类完整恢复文本（`static class X extends … implements … { … }`），外围文本内对折叠成员的引用（类头/字段/throws/方法体/子类间互引）按源码拼写重写（复用枚举折叠的名字重写投影机械）。
- M1（五子）与 M2（单子）家族整 jar 重编 `javac --release 8` 通过、运行与基线逐字一致（`hi`/`ok`、`7`）；分离家族呈现（逐类单类输出）与匿名/枚举投影通道逐字不变；孙代（`Inner$Leaf`）保持子类文本内池拼写（登记）；非静态成员类不做（后续片）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：直接静态成员类/接口按嵌套声明折叠呈现，折叠作用域内引用为源码拼写。

## Impact

根 crate `src/member_inner.rs`（家族扫描多子化）与 `src/facade.rs`（投影装配）及测试；复用枚举折叠投影与成员读取既有机械，无新跨类机制（只读本快照）。A16/成员计费沿用。分离呈现与投影通道零回退。
