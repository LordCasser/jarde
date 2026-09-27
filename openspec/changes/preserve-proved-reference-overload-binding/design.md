## Context

见 [proposal.md](proposal.md) 与 [EM-11 回放](../../evidence/java-syntax-2026-09-27/em11-overload-binding/report.md)。`jarde-java` 的 `invocation_argument` 已从物理 Methodref 获得目标参数类型，并有同型、`Object`、`null` 与封闭数组上溯路径；`ArrayList → List` 目前落入“无安全引用转换证据”的拒绝。`class_source` 对相邻泛型重载声明还会独立触发 `generic_call_binding_unproved`，本项只解决调用正文的绑定。

## Goals / Non-Goals

**Goals:** 在有确切引用上溯证据的调用参数位置沿既有 Cast 表达式和来源路径固定目标，先覆盖 Java 8 `ArrayList → List` 与本样本的同类/继承重载；负例继续拒绝。

**Non-Goals:** 方法 `Signature` 泛型投影、合成访问器/Smali、任意 classpath 的完整 Java 子类型求解、赋值/返回位置的通用引用上溯、向下转型和动态分派重建。

## Decisions

1. **沿既有调用消费入口处理。** 参考已有数组上溯路径，先证明呈现类型到目标参数类型的安全关系，再用 `cast_argument` 固定源级参数类型。cast 挂载原实参来源及调用 BCI，不改变生产者、IR 或物理 Methodref。单纯放行裸 `new ArrayList()` 会重新绑定到 `ArrayList` 重载。
2. **类型证据只取本次已选环境。** 对输入类使用现有有界、只读类头与直接父类/接口关系，按需遍历并计费；缺项、歧义、预算停止都不猜。Java 8 标准平台的 `java.util.ArrayList implements java.util.List` 可作为受版本和运行环境约束的内置事实，类似现有数组内置关系；仅在标准平台语义可确认、没有外部覆盖时使用。初片不新增公共类型解析服务、下载依赖或扫描宿主 classpath。未来扩大平台关系时仍须保持事实来源可审计，不能把本次单对关系当成任意外部层级许可。
3. **源级目标另行核对。** 在已选接收者层级可见的同名、同参量候选中，确认目标声明可被 Java 8 源级重载规则唯一选中；否则保持拒绝。`null`、数组、primitive、特殊调用原路径不受此证明替换。调用参数非同型时写目标参数 cast，即使当前候选碰巧不歧义，也避免未来候选或文本局部类型影响绑定。
4. **仍以完整源码验收。** 保留 [回放脚本](../../evidence/java-syntax-2026-09-27/em11-overload-binding/replay.py) 的原/JADX/Jarde 源码、哈希、Java 8 重编与 `-Xverify:all` 输出。`ArrayList → List` 正例必须让当前两处拒绝变为运行相同；未知层级与可能失败的 cast 为负例。`generic_call_binding_unproved` 的声明拒绝可仍显示在注释中，不能把这项债务误计为调用正文通过或失败。

## Risks / Trade-offs

- [平台事实与运行环境不符] → 严格限制为 Java 8 标准平台与无覆盖配置；其它环境维持拒绝。
- [证明了上溯却改绑到另一重载] → 把目标声明唯一性与完整源码运行对照列为独立门，不只检查 cast 文本。
- [跨类读取放大工作量] → 只读当前调用必要的层级头，复用既有预算/取消与物理身份，不建全局 classpath 图。
- [与泛型声明缺口混淆] → 报告和任务分别跟踪调用正文、声明投影；本 change 不放宽后者。
