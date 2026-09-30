# 标注循环巡查：label break/continue 与尾段覆盖（2026-10-01）

控制流账本未见标注跳转的正面 JVM 取证（主线 `1c920ca1`）。固定转录 [fixture](fixture/)（L1/L2/L3，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）；恢复文本与运行输出在 [results](results/)。

## 结果矩阵

| 场景 | 主线 Jarde | 行为 |
| --- | --- | --- |
| L1.labeled：`continue outer` + `break outer`，外层体无尾随语句 | 恢复；`continue outer` 呈现为内层 `break`（该形状下语义等价的简化）；`break outer` 用合成标签 `jarde_loop_10` | 重编一致 |
| L2.contWithTail：`continue outer` + 外层体**有尾随语句** | 恢复且正确呈现 `continue jarde_loop_10`（等价简化的反例被正确处理）；但尾段链式 append 的 `pop`（BCI 66）被引为 `@bytecode` 残留 | 重编一致（`00,10,`） |
| L1.plainNested / L3：无标注嵌套与普通循环的链式调用语句 | 零残留完整恢复 | 一致 |

## 发现（两个呈现质量缺口，正确性均保持）

1. **合成标签名**：`jarde_loop_10` 而非源码式标签名（`outer`）。标签在 classfile 无名字事实，命名属呈现选择；现拼写暴露内部合成痕迹，降低可读性与与 JADX 输出的可比性。
2. **标注循环尾段的 pop 覆盖**：`continue label` 循环的尾随语句段，其链式调用尾 `pop` 不被区域走查拥有（普通循环/直线体的同形态已覆盖，见 L3 对照）→ 输出残留 `@bytecode` 引用标记。文本仍可编译、行为一致（纯诊断残留），但违背"成功恢复无引用残留"的呈现目标。

## 处置方向（`recover-labeled-loop-tail-coverage`，串行排在 array-slot-retype 后）

尾段走查接受"非 void 调用 + 紧随 pop"（复用 17a 的 `discarded_call_pop` 判据，从 TWR 体检查点泛化到标注循环尾段）；标签命名改进（源码式拼写，如按锚点 BCI 语义化或 `loop`/`outer` 简名）作为同一呈现切片的次要项。两处均为呈现层，证明层零改动。

原 class 为行为基准；JADX 参照不作为语义正例。
