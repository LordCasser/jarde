## 1. 局部作用域规划

- [x] 1.1 增加无 LVT 的 Java 8 正向 fixture，分别覆盖 catch 块内自包含局部与 try/catch 后读取的共同局部；核对 fixture 可由 javac 编译、原 class 通过 `-Xverify:all`，并冻结输入 class SHA。root 独立用 `javac --release 8 -g:none` 重编[原源码](../../../tests/fixtures/p3-exception-scope/ExceptionScope.java)，与冻结 class 逐字节相同，SHA-256 为 `4ebd20bfc2146e02f8c77cd33ff472e090d440a0299304edf37226a86ee28fcf`；原/JADX/Jarde 完整类重编执行四项输出均为 `1,2,3,4`，当前 Jarde 无 `@bytecode`。本项只验收输入和这两个形状，不代表 1.2–2.5 的范围已完成。
- [ ] 1.2 让局部声明规划区分词法 owner、跨 region 可提升和证据不完整三种结果；定向测试验证 catch-only 声明不泄漏，完整的 try/catch 双路径赋值在共同作用域声明，且类型和 definite-assignment 事实一致。
- [ ] 1.3 在现有声明规划及 `Region::Try` 建造入口验证提升声明覆盖已知读写、catch/resource 子作用域与 fallback/未归属使用阻断，证据不足时先拒绝再建 AST；增加 catch/if/nested try 的输出与来源测试，确保离开子作用域后读取不存在，预算或取消停止时不提交部分正文。
- [x] 1.4 对无 LVT 的 `p3_nested_try`、`p3_typed_catch` 和 `p3_stated_rows::a_row_whose_range_holds_no_throwing_instruction_is_still_a_catch` 中相邻/嵌套 handler 复用 slot 1 的失败，按 clause 路径、handler entry store 和 SSA 定义—使用证明是独立绑定还是实际逃逸；恢复可证明的 catch，真实跨作用域使用仍拒绝。`p3_stated_rows` 当前误报 `local 1 escapes catch parameter scope at region [0, 1]`，其方法没有分支或循环，不能归咎于循环转移改动。不得仅因 `RegionPaths.catch_parameters` 的 slot 相同就报逃逸，也不得无条件拆分同槽的 try/catch 汇合局部。先固定现有错误文本与原/JADX/Jarde 对照，再做最小规划改动。
- [x] 1.5 用已冻结的 `LoopTryHandlerEntry.loopTry(II)I`（原 class SHA-256 `79254ba10b17846830ed420a8c13451a92630209edef9cb5d977fb6765e77f38`）闭合跨异常区的计算写入：BCI 1 的外层初始化、BCI 13 的 `result + maybeFail(...)`、BCI 19 的 catch 写入与 BCI 27 的循环后读取属于同一局部。复用现有表达式/单次消费与 Region/SSA 证据，在声明规划时证明可呈现写入及 definite assignment；正常与真实抛出两条路径原/JADX/Jarde 整类 Java 8 重编和 `-Xverify:all` 输出均为 `normal=6`、`caught=0`。不得以“写入不是直接整数常量”为唯一拒绝理由；若计算 producer 被引用或任一写入不能呈现，依赖切片仍完整拒绝，不能通过给局部任意默认值或移动调用出 try 来放行。该任务与 1.4 的 catch 参数同槽绑定分开验收，且不自动勾选 1.2、1.3 或整个 change。

## 2. 原子拒绝与行为验收

- [ ] 2.1 为保护区 fallback、异常边不完整及嵌套/共享 handler 增加负向 fixture；核对拒绝闭包包括相关定义、handler、汇合/transfer 和区域外 consumer 的 bytecode/origin，且正文不出现 catch 内声明后在外层读取的越界名称。
- [ ] 2.2 针对拒绝闭包与独立可恢复 sibling 增加边界断言，确认与局部无依赖的结构可保留、依赖切片完整拒绝，mixed/fallback 诊断和 source-map 可定位。
- [ ] 2.3 使用固定 fixture SHA，在新 CLI 对原类、JADX 完整类和 Jarde 完整类进行 javac、`-Xverify:all` 与 normal/异常输入执行对照；合法形状逐项比对返回值、trace、异常类型，拒绝形状只验收完整 bytecode/origin 覆盖及正确 refusal 标记，不把未生成 Java 计作语义通过。
- [ ] 2.4 在作用域规划、AST 校验及拒绝闭包的所有预算与取消点执行定向停止测试；确认结果遵循现有 Stopped/`content=not_produced` 契约且没有半个 try/catch 或 source-map 提交。
- [ ] 2.5 单独审计 `p3_try_local` 两项及 `p3_typed_catch::a_catch_type_of_zero_becomes_neither_a_catch_nor_a_finally` 的 `crosses a quoted fallback`：追踪 fallback 是否属于同一局部依赖切片，属于则修复区域证据或保留完整拒绝，不属于则缩窄拒绝范围；不通过删除守卫、降低旧断言或输出不可编译的局部来消除红测。上述失败在有/无本轮 `Frame::arm` 继承修复时相同，属于本项独立闭环。
