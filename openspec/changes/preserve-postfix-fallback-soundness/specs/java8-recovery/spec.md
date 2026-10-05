## ADDED Requirements

### Requirement: 旧值 store 证明失败时呈现文本不得可编译出不同行为

当 Java 8 方法的后缀自增/自减旧值 store 的值来源证明失败时，系统 SHALL 将与该未证明值语义绑定的整条语句并入引注区，使呈现文本（去注释后）不可编译、或行为与原 class 一致、或整方法响亮拒绝。

#### Scenario: 自赋陷阱三锚
- **WHEN** 输入为固定 `SA.postSelf`（`i = i++`）、`SD.postSelfDec`（`i = i--`）与 `SD.arrSelf`（`a[i] = i++`）的 Java 8 class 并恢复
- **THEN** 去注释呈现文本经 `javac --release 8` 编译若成功，运行输出 SHALL 与原 class 一致（5/5/102）；否则编译失败或整方法拒绝——不得出现可编译且输出不同（6/4/2）的文本

#### Scenario: 现有拒绝与健康形零回退
- **WHEN** 输入为 `SA.postOther`（现不可编译 fallback）与语句位 `i++;`/前缀 `++i` 健康形
- **THEN** 现有不可编译 fallback 与健康恢复 SHALL 保持（诊断文本逐字不变）

#### Scenario: 诊断不改
- **WHEN** 值来源证明失败
- **THEN** 诊断文本族（"the value at BCI N is the value local X held at BCI M…"）SHALL 保持，不发明新拒绝码
