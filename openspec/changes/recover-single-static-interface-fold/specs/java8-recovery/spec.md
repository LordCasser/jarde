## ADDED Requirements

### Requirement: 单直接静态接口子按嵌套声明折叠呈现

系统 SHALL 将外围类唯一的直接静态接口子（InnerClasses 0x0608 形态）按嵌套 `interface` 声明折叠呈现，域内引用为源码拼写。单静态类子、多子族与非静态候选 SHALL 与本变更前逐字一致；非静态单子保持不折叠。

#### Scenario: 单接口子折叠

- **WHEN** Y1 形（唯一 `interface StrFn` 子）家族三方 Java 8 重编
- **THEN** `interface StrFn` 嵌套呈现、`StrFn` 源码拼写，运行与原 class 一致

#### Scenario: 既有家族不变

- **WHEN** 输入为 M1/M2 多子或类形态家族
- **THEN** 输出与本变更前逐字一致
