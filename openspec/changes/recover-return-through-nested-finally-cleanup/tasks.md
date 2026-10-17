# Tasks

## 1. Establish the actual shape

- [x] 为封存的 `EX.nestedFin(I)I` 增加只读 Facts probe；记录 canonical 完整性、精确 rows/edges/throw sites、blocks、SSA values/phis、println effects/handler ownership，以及返回值和 rethrow identity 链。不改生产代码。
- [x] 记录当前 guard/region/builder 路径的首个拒绝。之后再决定现有 shape/Region/Builder 接缝是否足够，或是否必须增加最窄的 structured-cleanup tree。实现前记录判断及负边界。

## 2. Implement the bounded recovery

- [x] 使用实际 BCI/SSA/effect 证据实现最小 proof；证明保存的返回值、嵌套 cleanup 顺序、throwable store/reload、异常覆盖路径、完整 ownership 和 stop 行为。
- [x] 尽可能复用现有 SavedReturn 与 Try/finally 呈现。只有 task 1 证明扁平 `normal_cleanup` 无法表达已证明 tree 时，才增加私有 structured cleanup tree。
- [x] Builder 只有在正常及异常 cleanup tree 均完整呈现且 source origins 精确后，才原子 claim/publish。任何 unmatched edge、额外前驱、finally 非局部完成或无法呈现的操作均保留 fallback。

## 3. Pin positive behavior and adversarial boundaries

- [x] 为封存 EX 与独立完整 class fixture 增加永久 Facts/SSA/effects 与 full-source 正例。fixture target 只能有 constructor 和与 EX 同体的 `nestedFin`；注入只放在独立 recording-PrintStream driver。
- [x] 为缺失/变更异常 row、错误异常 reload identity、cleanup 顺序改变、无归属/额外 cleanup 入口和 ownership 不完整增加负变体；均须拒绝。
- [x] 增加 early/late budget、cancellation、deadline-stop 检查；确认停止时 Builder 不会部分发布，usage 计量保持正确。
- [x] 增加 guard 与 resource 回归门，确保本证书不放宽无关接受范围。
- [x] 用 javac 8 与 javac 23 `--release 8` 编译同一独立源文件并保留完整目标类。运行 compiler × JVM (8,23) × profile (`default`,`all`) 的 2×2×2 矩阵；执行精确 source/BCI/static ownership 检查、完整生成源码原样编译和 rebuilt-only runtime。
- [x] 运行完整矩阵 `n ∈ {2,0}`、`in ∈ {none, RuntimeException, Error}`、`out ∈ {none, RuntimeException, Error}`。断言 `IO` 恰好一次、正常时返回保存值、除零时 ArithmeticException 类型/消息正确、cleanup 异常的注入对象 identity 与覆盖优先级正确。不要声称运行时观察到原除零对象 identity；该 identity 由静态 SSA 证明。
- [x] 运行并固定 JADX 对照，保留其真实输出与编译/运行失败；不修改生成文本来制造一致。
- [x] 不删除无关方法，复核原完整 EX class 与双编译器完整 fixture；保留既有 finally-return/finally-throw 拒绝控制。

## 4. Exact-product and parent handoff

- [x] 记录精确实现 product commit SHA，核对它自己的 CI 收据确实对应该源码；覆盖 default/all、full-source、CLI/public entry、guard/resource、source/BCI 与项目要求的检查。不得用另一 product 或仅文档 head 的结果替代。
- [x] shared OpenSpec先取证和定义边界，再派发Luna私有实现；root审查、修正并在finThrow稳定后集成，产品4c6852已推送。精确product/自身CI/同一最终CLI完整对照见[仓库根交接](../../../handoff.md)及results索引，两个native seed亦已独立接受。
- [x] 仅在叶片证据完整后更新 parent remaining-positive matrix。集成后再运行 parent-wide final source/CLI/CI gates；exception-only 与 finally 自身 return/throw 继续留在范围外。

Root 已实际运行并审读只读探针，记录见 results/initial-facts-root-v1；表示选择与准确的首拒记录见 design 的 root review。其它任务仍需实际实现、验收与自身 CI。

2026-10-17 root实际focused-v3：25库测试/2公库API通过，sealed EX与双真实compiler输入均Structured、完整35BCI、default/essential/all/map-only文本及map一致；15完整canonical邻变体拒绝，已进入proof晚预算/取消/期限均保留usage且无Plan，公共晚预算无正文/映射。实现链已修正为store-local→load-local及load-stack→return-stack，原编译/SSA首拒和临时诊断原样保留。当前8/16，整类重编/运行、最终邻接回归、own CI与父最终门仍待实际验收。

root最终本地验收：fmt/当前CI Clippy/453库/2公库通过；独立类4配置/8整类重编/16 rebuilt-only×18=288case，加完整封存EX的4整类重编/8运行×18=144case，共432case逐字匹配。20个方法配置的ordinals/identity/全部BCI/映射UTF-8界限已核；独立fixture class身份与标准P5 BLAKE3指纹绑定，EX非nested方法文本与旧基线完全相同，finReturn fallback保留。固定JADX6次编译成功/12运行IOIO首错保留。另同一最终CLI重验finThrow16次×9=144case通过。当前13/16，精确产品自身CI、最终交付与父wide门待完成。

root已独立接受产品 `4c6852ac8c4ab9ce88f0716f4d2bd702e15684c5` 自身 CI39042611463：4 jobs / 52 API steps，两 seed 各376 targets、3691通过、0失败、98忽略、0 filtered，14项关键门均各一次通过。6收集命令/12raw逐字核验，workflow与当前生产源码绑定产品；详见 results/own-product-ci-root-v1。此时14/16，第二组本地 workspace 和最终父交付继续进行；上述13/16是此前阶段记录。

最终本地第二seed亦已接受，各376 targets/98批/3691通过/0失败/98忽略，两个seed完整inventory、4933来源及每target计数相同，target不存在。fresh FinallyCompletion拒绝控制8次完整原样编译仍恰好三处missing return，无runtime。当前15/16，仅父计划更新后的最终strict闭合未完成。

最终关闭：16/16。两个native seed、自身CI、完整正反对照与父级FY/FX/FO、29门均已root接受；父计划更新后OpenSpec1.11.0/1.14.0 strict均365/0，父7/7。精确产品4c6852，当前文档来源与产品生产源码一致；持续goal仍active，71/612不增加，未自动同步或移动归档。前述13/16、14/16、15/16均仅是历史阶段收据。
