## ADDED Requirements

### Requirement: A provable null resource is presented as a typed Java 8 header

当 Java 8 字节码以常量 `null` 初始化资源局部，且现有类事实足以唯一确定一个可在本类源码中声明的 `AutoCloseable` 资源类型时，恢复 SHALL 在完整证明正常关闭、异常关闭、抑制与重抛协议后把它写成 `try (T resource = null)`。正文、后续语句、调用次数和异常身份 MUST 与原 class 一致；合成清理 MUST NOT 写成用户 `catch`。

#### Scenario: Current-class null resource

- **WHEN** 当前类显式实现 `AutoCloseable`，资源是 `aconst_null` 初始化，正常与异常路径只在非 null 时调用当前类的 `close()V`，且其余资源协议可证明
- **THEN** 整类恢复源码 SHALL 可编译；该方法 SHALL 含当前类类型的 `try (… = null)`，正文执行一次、`close()` 不执行，运行结果与原 class 逐项相同，且不含合成 `Throwable` 的用户 `catch` 或未认领字节码引用

#### Scenario: The null resource's body throws

- **WHEN** 同一可证明的 null 资源头的正文抛出一个调用者持有的异常对象
- **THEN** 整类恢复源码 SHALL 抛出同一对象且不附加 suppressed 异常，`close()` 仍不执行；合成重抛不得写成额外的用户 `throw` 或 `catch`

#### Scenario: Adjacent non-null resources retain their semantics

- **WHEN** 相同恢复环境还含 nullable factory 结果、初始化抛异常及三个资源按 3→2→1 关闭并抑制异常的已证明 TWR
- **THEN** 这些方法 SHALL 保留可编译的 TWR 与原 class 相同的正文和初始化调用次数、关闭顺序、primary/suppressed 对象身份

### Requirement: A null header requires proof, not a guessed catch or type

常量 `null` 与局部存储本身不足以证明资源声明。候选区域的关闭协议或可声明类型不能证明时，系统 MUST 保留实际 BCI 的保守引用，不得猜测资源类型或把疑似 TWR 合成清理写成用户 `catch`。明确只是普通赋值后的用户 `try/catch` 时仍 SHALL 按异常表呈现其真实 `catch`。

#### Scenario: Ordinary catch after a null assignment

- **WHEN** `T x = null` 后的保护范围是现有命名 catch 规则已能呈现的普通用户 `catch`，不具备资源正常关闭与异常抑制协议
- **THEN** 系统 SHALL 保留该用户 `catch`，MUST NOT 发明 `try (… = null)`

#### Scenario: Resource shape has no provable declaration type

- **WHEN** 资源协议指向常量 null 头，但 close 属主或类声明不足以证明本次可声明的资源类型
- **THEN** 该区域 SHALL 保守引用并带真实来源，MUST NOT 写 `Object.close()`、虚构类型或用户 `catch (Throwable)`

#### Scenario: A near-resource has a broken cleanup protocol

- **WHEN** 候选 null 资源具有关闭链但异常抑制、重抛或范围中的任一必要证据不成立
- **THEN** 系统 SHALL 保留指名 BCI 的拒绝，MUST NOT 输出看似成功的 TWR 或把其合成处理器写成用户 `catch`

#### Scenario: Evidence and bounded requests

- **WHEN** 对相同合法 null 资源分别请求默认及完整来源，并施加预算或取消
- **THEN** 两种证据请求的正文 SHALL 一致；头、正常 close、异常 close 与抑制的真实 BCI SHALL 可追溯，预算或取消 MUST 按现有停止合同终止
