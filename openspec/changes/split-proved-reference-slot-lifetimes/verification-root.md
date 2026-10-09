# root 验收：已证明的普通引用槽生命周期

## 范围与结果

本片只处理无LVT、非参数/资源、普通且完整CFG中的异型引用槽复用。类型来自同一方法已有数组/常量/named frame事实；身份来自唯一SSA store owner、真实uses与段间正常路径。复用既有LocalVariable、NameTable、Declarations，不新增crate、parser、IR pass、层级推断或循环恢复机制。既有array/Ref-Int/monitor/LVT路径继续优先。

最终CLI2：SHA256 `f2ad93f5f024a0e1ec7d74e916839b17cac0f8db6fdf8c1a1eb62051f6dde33b`；源码快照与实际构建日志在 `results/candidate-cli-v2.json`、`candidate-v2-{reuse,build}.rs`、`cli-build-v2.stderr`。以该冻结工具实际完成32条完整类请求、32次完整重编、24次隔离运行及8条旧负例请求，共96子进程。root逐文件/双流校验296文件，无hash问题。

| 固定组 | CLI9基线 | 最终CLI2 | 判据 |
| --- | --- | --- | --- |
| 八类×两javac、no-debug | 0/16 | 16/16 | 完整类重编成功，`-Xverify:all`退出成功，stdout及stderr与原程序一致 |
| 不同名LVT | 8/8 | 8/8 | 完整源文本逐字保持，重编/运行结果保持 |
| 同名LVT | 0/8 | 0/8 | 完整失败文本与重编退出保持，不计成功 |
| 六族八个旧边界控制 | 保持旧恢复/拒绝 | 8/8文本、source map、命名/拒绝元数据保持 | 拒绝不计语法恢复 |
| 非零栈checkcast留栈、双javac | 原程序成功、完整渲染拒绝 | 两腿继续拒绝 | 完整文本、位置锚点、命名元数据保持 |

JADX与原程序的32腿完整源码基线均成功，且双流行为一致，已由root验证不可变归档；不是根据JADX测试断言或剥离成员推断。Jarde完整类能力由8/32增至24/32，尚未完成整个EM-20；完整LG仍有finalize分片及JADX Comparator基线失败，不能删成员后冒充闭合。

## 对抗审查发现并修正的问题

初候选CLI1虽能保持HeldUse拒绝源码，却错误发布额外别名。root实际SSA显示store9 ValueId(5)直接uses仅load10；load10产出Stack0 ValueId(6)，store19后invoke20仍消费它。只查store值的uses不足以证明死亡。失败的CLI1、32腿对照及元数据审计均保留，不作为最终验收。

最终reference-only检查沿已有load/store/dup/checkcast的身份转发继续查uses，仅已有replacement证明的平凡phi可以继续；不传播任意调用/运算结果，不为未知栈复制新增decoder。原数组证明保持。root另构造合法NonzeroHeldUse：checkcast13在Stack1产生ValueId(8)，store24之后invoke25仍读该值；真实双javac class及SSA转储证明不能假设Stack0总是栈顶。最终七个focused测试同时检查完整源码和恢复报告的aliased_names/diagnostics。

ArrayThenList/ListThenMap的独立真实SSA owner、类型、BCI与正常路径见 `results/implementation-notes.md`、`root-ir-audit-v3-*`。v1漏记读75、v2误将早期无BCI phi use判为跨界的审计失败原样保存；v3修正审计断言后成功，不修改产品来迁就断言。SSA replacement的uses迁移在既有 `Assigner::replace` 中完成，原代表值入口继续复用。

## 证据与复现

- 不可变原/JADX/CLI9基线：`evidence/{snapshot,negative-snapshot}.json`及对应tar，root核验脚本与JSON同目录。归档路径使用原根→archive根映射，原manifest不改写。
- 最终完整矩阵：`results/replay-candidate-root-v2.py`、`candidate-v2/manifest.json`、`candidate-v2-root-verification.json`。manifest SHA256 `875a5955ff93fb5299da94b75fd408a688c4956c8e7ae0234492af603fb6b1af`。
- 非零栈控制：`results/replay-nonzero-held-root.py`、`nonzero-held-{baseline,candidate}-v2`。两次打包jar字节不同、class与原Java相同；报告中provenance snapshot身份差异如实保留，位置/BCI锚不漂移。
- 逐字段负例审计：`results/audit-negative-semantic-v2.py`、`negative-semantic-audit-v2.{json,md}`。同输入8腿仅分析步骤及elapsed计数移动，HeldUse额外别名已消失；两条新腿的snapshot身份与计费差异分别保留。manifest同名source_sha256的含义分别为原Java与渲染Java，直接对比实际report.text字节确认一致。
- 所有Java编译显式空classpath/sourcepath，运行只用本次重编类；未删除拒绝成员、未借原jar编译或执行渲染源码。成功与失败各自原始双流/exit保留。

## 全仓与计费

新增证明真实收费：六行P5基准analysis_steps依次变为1360/2093/1121/10609/1121/1172，合计相对基线增加149；direct/shared均由17327变为17476。其它八个计费维度及输出字节不变，实际测量见 `results/p5-billing-measured-v3.*`、`p5-billing-delta-root-v3.json`。同型槽在已计费类型循环内退出，不进入身份转发闭包。

42份新增class使reader语料人口变为1011类/4296方法体/457handler/2693branch/8subroutine；语料指纹1882文件。仅按实际扫描重钉人口与指纹，未削弱断言。首轮P5计费、第二轮人口计数失败及修复后日志均保留。

完整门禁由 `results/run-local-gates-root-v3.py` 串行执行：两个固定seed各349targets、3311passed/0failed/93ignored；MSRV1.88、fmt、CI-exact Clippy、显式P3三测试及functional-constructor整类对照、strict OpenSpec、diff check全部exit0。实际argv/环境/双流hash/exit及空间检查在 `results/gates-v3/index.json`，root独立逐流核验和统计在 `local-gates-root-verification-v3.json`。本地显式Java检查使用已安装JDK；真实JDK25 oracle另由新提交的CI核对，不冒充本地已运行。

Cargo清理日志与前后空间在 `results/cargo-clean-final-v1.json`；最终提交/推送后的实际main CI结果另行附验。

## Actual CI acceptance — 2026-10-09

Exact code SHA `4fba93438acd444c4ce0f5d916a05f9af72caa28` CI run [37915062977](https://github.com/LordCasser/jarde/actions/runs/37915062977) completed success in all four jobs. Stable job includes both workspace seeds, ignored JDK25 instruction-boundary oracle, ignored P3 Java8 comparison and functional-constructor complete-family comparison; every step succeeded. Raw jobs/steps JSON is `results/ci-code-sha-4fba93438-success.json`; this closes task3.3 for this slice only, not EM20/71-unit parity.
