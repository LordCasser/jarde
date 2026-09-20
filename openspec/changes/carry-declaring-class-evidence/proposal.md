## Why

benchmark 的高频 `jre_declaration_class_not_in_run` 不能直接解释成依赖缺失：`engine::raw_facts` 已读取 driver 的 class Header，但 `MethodDeclaration` 和 `facade::recovery_facts` 只交接 member flags 等字段，丢掉 `this_class` 与 class flags。恢复层因此无法区分普通方法、interface default/static 方法，即使所需事实已在本次读取中取得。

## What Changes

- 从同一次已验证 Header 读取，把 driver 的 raw class name 与 class flags 随既有只读 declaration 载荷交给恢复层。
- 复用现有 `DeclaringClass`/声明规则，公开 recover_method 不再要求调用方重复提供该事实。
- 绑定物理定义及读取来源；停止或缺失证据时保持缺失，不从 entry 文件名、CP 引用或宿主环境推断。
- 用普通类、interface default/static、构造器与同名不同定义验证公开入口，并证明没有新增 Header/Body 扫描。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：driver 的声明类事实贯穿现有只读交接，公开入口可消费已读取的证据。

## Impact

前提为 P2 MethodIr/MethodDeclaration 与 P3 声明规则已交付。范围为 `jarde-jvm` 的 raw_facts/MethodDeclaration、根 facade 适配、`jarde-java` 现有事实及对应测试；不新增 crate、provider、全局符号表或第三方库。

不扩展跨类 Body、enum switch 表、InnerClasses、MethodParameters、handler 根策略或完整类源码。清除该诊断只证明交接补齐，不承诺语句率或 Structured 比例提高。关联分析见 [benchmark review](../../benchmark-review.md)。
