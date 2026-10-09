## Why

方法 Signature 单独获证并发布后，同类调用仍可能按擦除形实参发射，导致完整类不能编译：CallRelay 和 CallHold/ExceptionHold 已有独立重编失败。BoundOverload 还证明，恢复 relay(T) 会暴露字节码没有保留的源码重载转换，Jarde 与参考 JADX 均出现歧义；单靠物理目标绑定或参数未重写不足以保证源码正确。

## What Changes

- 将普通同类泛型调用作为一个完整里程碑处理：直接 T/T[]、已验证上界、多参数及宽槽、方法 binder 的直接代换、有限 relay 链、void 调用和返回消费。
- 复用同次最终 Program 与真实 Code/SSA，逐调用实参、receiver 和结果消费证明实际发射类型；按已发布 callee 声明闭合 caller 泛型头，而不是默认插入向未知 T 的 cast。
- 在现有 method publication 与 field publication 之间完成有界调用证明和原子源码提交；覆盖 Object() 后调用结果字段赋值及已恢复普通 catch 的 CallHold/ExceptionHold，保留初始化与异常顺序。
- 对封闭同类重载集合，以真正源码类型保留物理目标；支持有证据的无运行时检查的上转型 pin，修复 BoundOverload 的歧义。
- 冻结真实 Corretto8/OpenJDK23、debug/no-debug 四腿完整源码、JADX、Jarde 重编/行为/泛型反射和可靠拒绝控制，独立验收既有字段、构造器与 raw receiver 片无回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 泛型方法与构造参数投影须与同类调用消费位及源码重载绑定共同成立，支持封闭的实参与结果类型代换。

## Impact

前置主线为 d158989b，raw receiver 已交付且最新 CI 四项成功。影响 jarde-java 的既有同次 AST/调用事实与投影接口、根 facade 的 same-class inventory/publication、class_source 的泛型声明与消费位证明；不增加 crate、依赖、parser、IR pass 或全局类型传播。getter 对字段先发布的反向依赖、跨类/继承调用解析、任意泛型容器/wildcard 推断、复杂 raw alias/phi、this/super 委派和通用递归调用推断单独登记，不混入本片。71 单元账本中的泛型单元不会因该里程碑而自动标为全部追平。
