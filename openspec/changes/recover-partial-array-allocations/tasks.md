## 1. 固定可执行输入与边界

- [x] 1.1 从 `core-no-overload/` 建立最小永久 Java 8 fixture；固定主 class/helper/runner 源码、class SHA/Code、69 项原 class 与 JADX 对照、当前 jarde 整类失败阶段；由 root 统一更新 census/fingerprint。完整 72 项重载样本只作相邻边界。
- [x] 1.2 增加 `anewarray` 组件为基本/引用数组、`multianewarray` 前缀和完整维度对照；覆盖负/零/正尺寸、尺寸函数抛错、局部/字段/长度及数组后续消费者；非法 rank 或不能安全呈现的生产者单列拒绝测试。

## 2. 补足一个已有数组形状事实

- [x] 2.1 在 `array_creation` 从常量池描述符与立即数构造 `total_dimensions`，验证 rank 不变量；普通 `newarray`、`anewarray`、完整 `multianewarray` 保持既有路径，无新 pass 或求解器。
- [x] 2.2 在已有 `NewArray` 表达式、`presented_of`、`array_of_value` 和 `emit` 中使用同一总 rank，写出尾部空括号；保持尺寸表达式只按已分配前缀渲染一次、最终消费者上下文、已有来源/预算/取消。
- [x] 2.3 用真实 class 验证默认/all 正文一致、创建及尺寸来源、无非法 Java 语法或重复尺寸调用；缺少安全延迟绑定时完整拒绝，不能提前移动生产者。

## 3. 整类语义与 root 验收

- [x] 3.1 原样重编译执行永久 fixture 与 69 项核心输入，要求所有已授权正例零引用、源码可编译、原 class/JADX/jarde 的形状、trace、异常逐项一致；JADX 只有编译后才可作为执行 oracle。
- [x] 3.2 root 独立审读 rank/count 边界并重放完整类；复跑现有 array access、类型/局部/字段、共享 deferred 顺序回归，记录仍属重载数组协变等范围外债务。
- [x] 3.3 root 统一 census/fingerprint、fmt、适当 cargo 回归与 OpenSpec strict；仅在证据完成后勾选任务。
