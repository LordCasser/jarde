## Why

EM-20 的局部变量身份仍有明确的 JADX 正向差距：八个完整类经真 javac 8 / javac 23、`-g` / `-g:none` 编译后，同一槽先后存入不同引用类型。JADX 32/32 完整重编并运行一致；当前 CLI9 仅 8/32，通过的都是不同名称 LVT 腿，16 个无 LVT 输入全部重编失败。

## What Changes

- 对无 LVT、普通正常流、旧值已死的不同引用类型生命周期，以现有 SSA 定义/使用和 CFG 事实形成局部变量分段。
- 复用 `reuse::Plan`、`LocalVariable`、名称去重和声明规划，让各段分别按已有类型事实声明，避免数组或旧对象类型沿用到后段。
- 保持已有数组间分段、Ref/Int 分段、LVT 分段及守卫头部的既有证据边界；冻结全部源集、重编、验证运行及负控制。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 补充可证明的普通引用局部生命周期分段与不越界的呈现要求。

## Impact

前提是本次同一方法的 Code、帧、SSA、canonical CFG 与 Region 已准备。预计仅影响 `jarde-java` 的 `reuse.rs` 和已有类型事实读取的 crate 内可见性，以及一个聚焦测试家族；不新增 crate、IR pass、parser、类型推断系统或对外 API。

不处理同名 LVT、参数/receiver、资源头部、caught 值、未知/null 类型、跨段 phi、旧值跨段栈存活、后段返回前段或异常/call-context 流。完整 LG 的 `finalize` 与泛型失败单独排队。本片不宣称 EM-20 五项测试或完整 LG 已全部覆盖；其中两个 JADX 测试是 smali，另一个明确禁重编，必须分层记录。
