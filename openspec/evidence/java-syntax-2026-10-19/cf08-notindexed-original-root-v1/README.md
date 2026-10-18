# CF08：TestNotIndexedLoop 完整原始类对照

Root 已接受原始类/API 与完整生成源码矩阵。没有新增生产实现或 OpenSpec；现有循环、构造和局部类型机制足以处理此固定 Java 输入。CF08 与 71 单元 / 612 文件的整项完成计数不变。

原始上游文件 SHA256 为 `dc08e939d5d5ab12c2f4a89e11b266a7e9592ca6289e700241a5eca2f29980b5`。完整保留外层测试及目标类的 constructor、test(File[])、check()，没有删 helper 或修补反编译文本。当前生产 CLI 为 `96e534b0000fe91c1d34a282919479a97eb4706063501d188cdff010c4640366`，其生产来源与已有主线产品 ef4fbef5 的验收绑定不变。

| 范围 | 实际结果 |
| --- | --- |
| 原始类 | debug/no-debug × JDK8/JDK23：4 VM / 20 观察 |
| Jarde API | 8 cells / 24 structured 方法；default/all 文本与 raw map 相同 |
| 完整生成源码 | 10 源格 × 两编译器 = 20 次尝试 |
| Jarde 成功编译 | 8 次 JDK23；16 VM / 80 观察，同原始 |
| JADX 成功编译 | 2 次 JDK23；4 VM / 20 观察，同原始 |
| 框架版本失败 | 10 次 JDK8 编译失败；raw 均指向 harness major55 对52，无生成 class、无 VM |

两 collector 共71条实际命令、142份 raw 流，wrapper/audit 流另存。五个受控用例是 null 数组、空数组、前缀后命中 f、未命中生成 h、null 元素异常。每次驱动使用独立空 cwd，前后检查；只创建 File 路径对象，不创建 a/f/h 文件。parentless target-only loader、driver-only 应用 classpath、正向和10个生成错根 CodeSource 拒绝均通过。原始错根控制另计。物理方法/字段/access/Code/BCI、完整 source copy 及工具/来源闭合已独立复开。

驱动没有执行 check() 或外层 JUnit 方法。Root 另核全部八份 Jarde 文本符合上游此处的“一处 while(true)、无 for”谓词；这不冒称运行 JUnit。普通 String.equals(Object) 的 Object 展示 cast 单列质量债务，不能通用删除 descriptor 包装。

[capture.tar.gz](capture.tar.gz) 保存原始收据、raw、完整 source/class、准备与审计所有留存版本和上游许可快照；[逐成员库存](capture.tar.gz.inventory.json) 与 [root 复开](capture-review.json) 为283个 payload成员加1份内嵌库存。旧 auditor/备忘错误与更正均保留，正式权威是 root audit 和 canonical root-review。归档准备 README 中“20源格”和“生成阶段包含提取”的表述不准确：实际10源格、20编译，提取属于 baseline20命令；未修改历史稿。

baseline 收据 SHA：`56276e1841a38d4929826cfb52440ef8ef88d07efae07149a06a6626ae10c967`；generated：`8e57bf803579a692df08388e168f336fa44cb072620fbc81f1daa517e2253b4c`。外部大型工具保留 pins，不复制安装目录。后续继续固定 EndlessLoop 与多入口形；JVM 构造样例不能替代原 Smali/DEX 输入。
