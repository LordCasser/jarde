## ADDED Requirements

### Requirement: 值来源、依赖链或多消费者证明失败（旧值 store / copy / 依赖链 / 多消费者四诊断族）时呈现文本不得可编译出不同行为

当 Java 8 方法的后缀自增/自减旧值 store 的值来源证明失败时，系统 SHALL 将与该未证明值语义绑定的整条语句并入引注区，使呈现文本（去注释后）不可编译、或行为与原 class 一致、或整方法响亮拒绝。

#### Scenario: 自赋陷阱三锚
- **WHEN** 输入为固定 `SA.postSelf`（`i = i++`）、`SD.postSelfDec`（`i = i--`）与 `SD.arrSelf`（`a[i] = i++`）与 `FS.compoundSelf`（`i += i++ + 1`）的 Java 8 class 并恢复
- **THEN** 去注释呈现文本经 `javac --release 8` 编译若成功，运行输出 SHALL 与原 class 一致（5/5/102/11）；否则编译失败或整方法拒绝——不得出现可编译且输出不同（6/4/2/6）的文本

#### Scenario: 现有拒绝与健康形零回退
- **WHEN** 输入为 `SA.postOther`（现不可编译 fallback）与语句位 `i++;`/前缀 `++i` 健康形
- **THEN** 现有不可编译 fallback 与健康恢复 SHALL 保持（诊断文本逐字不变）

#### Scenario: 实例字段复合赋值锚点（copy 族触发）
- **WHEN** 输入为固定 `BF`（`this.flags |= 1 << bit` void 与 `BG.ienable2` 值消费形——receiver dup 跨 getfield+putfield，诊断 "the copy at BCI 1 has no proved local assignment"）的 Java 8 class 并恢复
- **THEN** 去注释呈现文本经 `javac --release 8` 编译若成功，运行输出 SHALL 与原 class 一致（true/false/true/false/3 与 false/2）；否则编译失败或整方法拒绝——不得出现可编译且位操作被静默丢弃的文本

#### Scenario: 字段数组副作用存储锚点（依赖链族触发）
- **WHEN** 输入为固定 `AD`（`this.elems[this.size++] = arg1`——字段数组 dup_x1 副作用下标舞蹈，诊断 "the dependency chain from BCI N to final consumer M is not bounded"）的 Java 8 class 并恢复
- **THEN** 去注释呈现文本经 `javac --release 8` 编译若成功，运行输出 SHALL 与原 class 一致（`x/y`）；否则编译失败或整方法拒绝——不得出现可编译且存储静默丢弃的文本（`null/null`）

#### Scenario: 实参消费位锚点（链实参与方法实参）
- **WHEN** 输入为固定 `PC`（`sb.append("n").append(t++)` 链实参位与 `list.set(idx++, "X")` 方法实参位）的 Java 8 class 并恢复
- **THEN** 去注释呈现文本（单方法隔离）经 `javac --release 8` 编译若成功，运行输出 SHALL 与原 class 一致（`n5:6` 与 `1`）；否则编译失败或整方法拒绝——不得出现旧值被吞的可编译文本（`n:6`、`0`）；`return x++` 返回位的整方法响亮拒绝 SHALL 保持

#### Scenario: 多消费者局部锚点（第 4 族触发）
- **WHEN** 输入为固定 `NI`（main 中局部被单表达式消费 4 次，诊断 "the saved producer at BCI N has M consumers, so one local binding cannot prove its execution count"）的 Java 8 class 并恢复
- **THEN** 去注释呈现文本经 `javac --release 8` 编译若成功，运行输出 SHALL 与原 class 一致（`3/10/5/true`）；否则编译失败或整方法拒绝——不得出现可编译且整条表达式静默丢失的文本（无输出）；2 次消费单调用形（`NJ`）的完整恢复 SHALL 保持

#### Scenario: 诊断不改
- **WHEN** 值来源证明失败（任一诊断族）
- **THEN** 诊断文本族——四主族（"the value at BCI N is the value local X held at BCI M…"、"the copy at BCI N has no proved local assignment…"、"the dependency chain from BCI N to final consumer M is not bounded…"、"the saved producer at BCI N has M consumers, so one local binding cannot prove its execution count…"）及其级联伴随行（"the saved producer at BCI N has no bounded final expression consumer"、"the value at BCI N comes from an Other…"、"the value at BCI N was produced by a saved declaration this run could not commit"、"the instruction at BCI N is not part of the provable subset"、"the value at BCI N is the entry state of stack depth N…"，[诊断普查](../../../../evidence/java-syntax-2026-10-05/diagnosis-census/README.md)）SHALL 保持，不发明新拒绝码
