## Context

动机见proposal，已执行证据见results/initial-facts-root-v1。root在当前0cf8/4c6852源码上运行两907B封存输入：Complete，两块`[0,10)`、`[10,19)`，一条catch-all `[0,11)→10`，仅异常边0→10，无正常边、无自边。可抛点2/6/9的handler仅row0，cleanup调用13与重抛18的handler为空；绑定store10不可抛。handler的Stack0 phi合并三个真实Caught值，随后store10写Local7，load17产生新的Stack10，throw18读该Stack10；不能直接比较Local与新Stack身份。

旧finally-copy候选确实成立，但实际generic proof把before-handler末尾当return/load，推出cleanup_start0，随后save.checked_sub(1)为None而拒绝。resources与examine均报告finally_copy@10。这里不存在normal cleanup副本；旧诊断的“正常路径重复代码”只是候选描述。普通exception-only catch实际也拒绝cleanup grammar（它只接受静态整数增量），且普通catch呈现不能替代本次finally恢复。

## Goals / Non-Goals

**Goals:** 支持有界、完整、直线的throw-only主体与落穿cleanup；同时保留try调用先抛错、原显式throw及cleanup调用覆盖三种路径。所有物理指令归属一次，保持caught phi到store/local/load/throw的完整链；最终通过既有Try/finally AST呈现。

**Non-Goals:** 不把Void误作“没有正常完成”，不制造normal cleanup副本，不恢复finally自身显式return/throw，不推广为任意异常图，不扩其它CF单元。现有FinallyOnce异常专用catch仍保持其既有文本与拒绝范围。

## Decisions

### 以真实完成类型复用现有Finally

在既有FinallyCompletion增加`Throws { throw_bci }`，明确protected body不存在正常完成且终结于已证明的athrow。Shape::Finally的`normal_cleanup`改名为`cleanup`，表示被选作源级finally本体的一份实际指令区间：既有形选正常副本，新形选exceptional handler内去掉绑定与重抛尾的区间。只更名该Shape字段，不改仍准确表示正常副本的其它结构字段。

不选择给私有Plan增加标志却公开伪装成Void/SavedReturn：这会暴露不存在的return或normal copy。也不增加平行Shape/solver/通用完成仲裁框架；新的语义值在现有完成枚举中表达即可。无新依赖。JADX为GPL源码参考，项目为MIT/Apache-2.0；只借鉴捕获身份和延后构造region的思路，本实现依据本引擎Facts/SSA独立编写，不移入其实现代码或依赖。

### 先闭合有界证据，再允许呈现

私有proof在既有FINALLY准入中、generic copy拒绝之前尝试。要求单catch-all、完整且无遗漏的两个真实块、有界instructions/phis/values、唯一异常边、无正常入口/出口；范围从实际row与操作推导，不允许类名、方法名、BCI字面量识别。body保留原始athrow；handler为store/完整cleanup/load/同值rethrow。保护区可包含不可抛的绑定store，但不得包含任何cleanup操作。

逐项证明所有may-throw sites及精确handler集合、Caught输入及重抛SSA身份、cleanup表达式的完整consumer/类型/可呈现性、body/handler物理分区及全块唯一ownership；缺操作/边/值均拒绝。捕获Stack0 phi之外，只允许handler Local槽上的已消去且无人使用的冗余phi：其每个真实throw-site输入均解析为同一入口块、同一Local槽的Entry参数值；非平凡合流、活跃值或其它定义仍拒绝。这来自独立参数读取变体的实际Facts首拒，不引入phi求解器。扫描、比较与构造按现有Budget付费并在成功前最终poll。候选未成立时沿原逻辑继续，不放宽旧finally-copy或ordinary catch grammar。

Region保留现有Guard入口，不为Throws打开SavedReturn的loop terminal或normal continuation授权。Builder用body_range保留原throw，用cleanup区间构造现有Try.finally_body，不追加return、不输出捕获参数或机械rethrow。Throws也建立完整finally checkpoint，包含declare_at、body、cleanup、最终push及来源：任何失败恢复全部临时状态；只有完整无fallback且声明/来源闭合时才发表。

### 分开验证完整原类与独立正向类

两封存原FinallyCompletion的七方法全部保留。本次只改善tryThrows，finallyReturns/finallyThrows仍是范围外红项；原样整类编译应保留它们实际错误，不删除方法或手修生成文本，也不以整类红项否定目标方法静态恢复。

另从零定义完整ExceptionOnlyFinally：constructor、mark、目标方法、clinit及字段均保留，目标try/cleanup与封存形同体。mark内部经PrintStream记录/注入，目标方法本身不加hook。独立driver对try-call/cleanup-call各none、RuntimeException、Error的九组合核`78`恰好一次、trace、类型/message及对象identity。原始两javac输入×双JVM作oracle；default/all完整生成类原样用javac8/23编译，再仅运行重建classpath。还需固定JADX1.5.6同输入的新对照，旧漏mark7记录仅为历史。

## Risks / Trade-offs

- [enum与字段命名影响消费者] → 搜索完整匹配面，所有构造/Region/Builder及测试显式处理Throws；既有finally正文/物理来源须保持，不能全局替换其它normal_cleanup字段。
- [同slot不是同异常] → 分别证明caught-phi→store-local、store-local→load-local及load-stack→throw-stack，保留所有真实throw-site输入。
- [cleanup调用抛错被误当拒绝] → 精确证明其无原handler保护；AST自然实现覆盖，driver显式测试RTE/Error，不增加仲裁机制。
- [reader有效被误当JVM或源码有效] → 邻变体先核实际IR完整性，必要时双JVM严格验证；源码编译、runtime oracle和静态证明分别记账。
- [局部成功冒称产品完成] → early/late停止、完整来源、旧CF16及finThrow/nestedFin回归、本地workspace与精确产品自身CI分别验收；71/612不因叶片增加。

## Migration Plan

root独占共享Git/编译/JDK/JADX/CLI，Luna只在私有副本准备实现和测试；root审查后串行集成。所有Rust构建保持5GiB余量、target1GiB上限、每秒进程组守卫、默认Rust栈，并在finally路径exact cargo clean。先采当前CLI/JADX红基线，再集成、完整对照和精确产品自身CI；失败raw保留，未验收项保持待办。
