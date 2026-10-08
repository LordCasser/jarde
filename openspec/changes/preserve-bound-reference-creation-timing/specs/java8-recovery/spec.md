## ADDED Requirements

### Requirement: 绑定方法引用的创建时机

系统 SHALL 仅在同轮事实证明绑定 receiver 的引用语法不会新增或移动创建时 null 失败时输出 bound method reference。该条件 SHALL 同样约束无参数适配的直接引用；bootstrap/SAM/handle 类型匹配不等于非null证明。证明不足时 MUST 保留完整字节码、capture/consumer来源与明确拒绝，不得新增创建时异常或擅自延后capture。

#### Scenario: 无check工厂可捕获null
- **WHEN** 合法 major52 的虚拟handle工厂捕获Thread参数且没有创建时null-check，原类对null创建成功并在run时NPE，无论函数值直接返回或作为构造实参
- **THEN** 系统 SHALL 拒绝无证明的arg0::start输出；若输出可执行函数语法，其创建/调用输出 MUST 与原类逐字一致；不得以可编译替代时机证明

#### Scenario: 已证明的非null接收者
- **WHEN** receiver是同轮确认的真实实例方法entry this，或完成分配/稳定move chain、受既有receiver-tail证明约束的值，或直接非null常量，并且其余函数证明成立
- **THEN** 既有可恢复方法引用 SHALL 保持合法恢复及行为；静态方法/构造器引用、不依赖绑定receiver的lambda SHALL 不退化

#### Scenario: 静态local0和不完整证明
- **WHEN** 静态方法的第一个参数占local0，或receiver来自不完整/未证明值链
- **THEN** 系统 MUST NOT 将该值当作entry this非null；整个相关表达式 SHALL 可靠拒绝并保留origin

#### Scenario: 原有显式check边界
- **WHEN** javac在工厂前写入Objects.requireNonNull或getClass等显式check，而构造/捕获仍不满足已有闭合与effect证明
- **THEN** 系统 SHALL 保持可靠拒绝与该check的effect；不得删除未证明check来获得方法引用
