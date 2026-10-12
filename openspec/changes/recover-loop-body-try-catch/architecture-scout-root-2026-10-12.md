# 循环体 try/catch：实现前范围复核（2026-10-12）

当前状态是只读架构侦察。没有复跑新 CLI、插桩或实现，不把 10-05 的拒绝记录认作当前产品缺陷，也不关闭现有任务。

Root 阅读了当前 `region::recover`、`proved_loop_catch_joins` 和固定 DV 源码、旧完整诊断，以及本地 JADX `TestTryCatchInLoop`。Atlas scoped search 与 explore/resume 均限于 region.rs；最终返回 local complete，源函数没有截断。Atlas 普通方法名的调用解析不能替代 Rust 接收者类型核准，例如 collect/protected 的同名边不作为架构证明。

## 已核准的事实

- `region::recover` 先执行 fragmented 认证，再取得 `proved_loop_catch_joins` 的认证边，合入既有 `catch_joins` 后交给 `irreducible_blocks`。因此本次应先定位已有认证缺失的具体条件，不能直接删除 CFG 边来让归约通过。
- 普通认证当前要求 exception-only handler、唯一普通后继 join、声明行与 canonical 行完整映射、具名且非空保护区、join 属于循环且不是 header；完整异常表区间重叠的块不得是 header/环外，须被 header 支配；每个 protected block 的普通路径须到 join。暂未执行 Q-i，所以还不知道 plainFor 究竟失败在哪一个合取项。
- 提案的 Q-i 把 `plainFor` 标在 DF.java；实际固定源方法在 **DV.java**。实施时应按 class 身份和方法 descriptor 定位。
- 旧 DV 诊断中，plainFor、whileLoop 是 irreducible；forEach 是 `local 2 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write`。三形存在不同处理门。新复跑须分别记录第一处拒绝，不能把 foreach 的局部变量问题算成同一个归约问题。
- JADX 最直接的既有代表测试是 `loops/TestTryCatchInLoop.java`：无限 while 的 try 内调用及 break、catch 后迭代；源码 check 验证计数为 3，JUnit test 断言 catch 与 break 各一次。此处只审阅断言，未执行该测试。它与有限计数循环的 body-only catch 不是完全相同形状，基线需要同时保留。

## 下一步的最小闭环

先在串行工具链空闲时，用当前 CLI 复跑固定 DF/DV 的整类原/JADX/Jarde，保留 debug/none 的 class 身份、物理异常表、源码与生成源码、编译失败的 no-runtime 记录、成功运行的异常/效果/集合状态与来源。对 DV.plainFor 的拒绝按 Q-i 逐项记录 handler/row/header/join 与具体普通/异常边；Q-ii 使用当前原图和既有认证边集合验证归约判据，Q-iii 单独记录 foreach 的局部 scope 阶段。

优先候选仍是扩展普通 `proved_loop_catch_joins`。fragmented 认证服务于拆段 outer/inner catch 和 induction-update，不能为了简单单行 catch 放宽其特化门。单 catch、单异常行、body-only 保护区先行；是否需要有限增强必须由新同次事实决定。handler 普通前驱、跨 header 的保护区、交叉异常表仍拒绝。若 foreach 确实是独立 SSA 问题，应按完整目标另列实现任务，不用关闭一例代表整单元完成。

私有 Luna 侦察只作为索引，root 的以上结论来自当前代码与原始文件核对。71 单元/612 文件和 CF-18 完成数保持原记录。本笔记没有把新源码、测试或机制混入当前 List 泛型实现。
