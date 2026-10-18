## Why

固定原始 `TestNestedLoops` 已经恢复两层 String foreach，但仍使用合成头名并在正文复制到 s1/s2；JADX 已直接使用元素变量。真实完整源码与运行对照已证明现有表示可靠，现在补齐局部使用范围证明，消除这段已确定的呈现差距。

## What Changes

- 对元素转换、唯一局部写入和全部引用范围闭合的 foreach，将首句原目标提升为头变量，删除对应首句及确无其它用途的外提空声明；同时处理外层 Assign 和内层 Declare。
- **BREAKING**：上述已证明情形的投影源码不再保留合成头名和中间复制；变量已有名字、求值和异常顺序、其余正文及 raw 方法文本/映射保持可靠。
- 循环外可观察、已初始化、有其它写入、作用域冲突或缺证者保留已交付 typed 头与原绑定；真实停止不发布部分投影。
- 显式演进已同步的 `Typed List foreach preserves the existing body binding` 要求，不能在旧要求仍无条件保留正文绑定时暗中删除。

### Prerequisites and Non-Goals

基底为已交付 e69a3494c 产品、dadf70240 文档主线；自身 CI39112513084、两 fresh native、最终85ca CLI的20 API与完整原始/控制矩阵绑定均已接受。原始目标45条指令/119B，两头锚点13/42，next17/47；外层空声明和首赋值同BCI25，不允许按BCI或名字猜测。完整证据见前片 results 与 `cf11-foreach-original-bindings-analysis-root-v1`。

本片不新增循环识别、全局作用域/活跃性求解器或类型推断，不扩 raw/wildcard/一般 Iterable 范围，不清理无关 iterator 声明、Object 调用包装或既有 AST 计费预扫债务，也不声称整条原始文本黄金、CF11或71/612已经完成。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：有完整局部使用和词法可见性证明时允许将正文元素绑定提升为 foreach 头，缺证时继续保持已交付 typed 表示及事务/来源契约。

## Impact

限于现有 facade 泛型源码事务、jarde-java opaque AST 的窄绑定事实与投影，以及必要永久回归/完整对照。复用同次物理 slot/SSA 和原 ForEach，不新增 crate、依赖、公有序列化模型、通用 scope 框架或第二次 Reader/SSA。Luna 私有实现，root 审查与实际编译、三方生成源码重编和隔离运行验收。
