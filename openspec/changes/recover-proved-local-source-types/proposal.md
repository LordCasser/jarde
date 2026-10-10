## Why

原样重放 CF12 五份上游 Java 测试后，char switch 的局部变量从 charAt 的 C 返回描述符退成 int；无 default switch 的 null/String 局部从首个 null 写入退成 Object。两者在准确调用描述符处被拒绝；完整输出分别有部分运行差异和编译拒绝，证据见 `openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/`。

## What Changes

- 在既有局部声明类型决策中使用同次 SSA/operations 的有限全写证明，保留有准确 producer 类型的 char 局部，以及 null 初始化、其余写均为同一准确引用类型的局部。
- 复用现有声明、调用转换、switch 标签和来源输出，保持 int→char、Object→任意引用的未证转换拒绝。
- 原样重放真实上游完整 class、check()、JADX 和 Jarde 输出及近邻反例，分别记录编译、运行、来源和预算/取消结果。
- 应用前保存并独立核验实际基线、完成架构审查；root 以 Luna 私有补丁实现与验收。条件 fallthrough、嵌套根常量名投影和跨类别名另行处理，不能靠本片关闭整个 CF12。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 局部变量类型由完整可证明写入决定，保留 JVM frame 无法区分的 char 与 null 初始化引用类型。

## Impact

限 jarde-java 既有声明类型决策与实际回归；无新 IR、Frame、pass、类型推断框架、依赖或 API。71 单元/612 文件分母不变；历史首片和失败 raw 保留。资源沿用户批准的 5 GiB machine free/target 1 GiB、一秒进程组守卫和完成清理。
