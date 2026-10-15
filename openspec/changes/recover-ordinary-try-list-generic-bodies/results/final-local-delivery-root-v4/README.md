# 整数 carry 依赖闭合修复：最终本地验收

f014/v14 的完整工厂 receiver 反例曾编译、运行成功却输出空效果；双编译器原结果为 `false|8|MVT`、`true|7|MVTA`。本次修复先规划已证明的整数保存点，再处理其上游依赖；保存声明失败恢复原多消费者 primary 和整方法拒绝，预算/取消继续传播。

最终 CLI 为 59590432 bytes，SHA `cd5e52dd45234b45e026af0117dae5f77bf3ef2a1316181bbbaf8c672c95679d`。v17/v18/v19 逐字相同；后两次只有测试、fixture 与精确计费 pins 变化，两个 source-delta 审计保留完整差异。

- focused-v19：642 passed / 0 failed / 4 ignored；fmt、CI exact Clippy、OpenSpec 1.11 strict 367/0、421 Java 库、178 reader 库及原 ICN/List 控制通过。真实 carry 证书上的三个保存失败出口、实际准备阶段 budget/ledger cancellation 和原晚期证明停止均有断言。
- 工厂/this/parameter receiver：双 JDK、default/all 四个完整十方法类重编译运行，六行 oracle 逐字同原；40 方法 maps 独立审计通过。生成 Java 未手改。
- receiver 赋值拒绝控制：四个完整两方法类保持原三消费者原因、ExplanationOnly 和全部 13 个物理 BCI，无幸存 return。带正确原 Runner、原 helper classpath 的四次真实 javac 均仅因 `jarde_refused_body()` 失败，不执行 runtime。早期 collector 的未使用 Runner 副本错误保留并单独说明；它未传给第一次 compile，第二次明确使用正确 Runner。
- 原矩阵：baseline 十七方法/45 行、admission 十三方法/33 行，16 个 Jarde 完整类 cells 全部编译运行同原；JADX baseline 八次编译失败/no-runtime、admission 八次成功如实保留。原十六方法时序 probe 四 cells/64 maps/12 行通过；BI/ICN 二十次 render 的正文、质量、来源和身份与旧产品一致。
- 多维数组下游猜测未成立：完整七方法类四 cells/28 maps、两行 oracle 为 `false|103|DTC`、`true|102|DTAC`，均逐字同原。已有 NewArray 路径支持该操作，未新增 solver。其原 Java/Runner 同字节副本在 [original-inputs/downstream-multiarray](original-inputs/downstream-multiarray/)，由独立 review 的 supplemental pins 校验。
- fresh workspace-v10：两个 seed 各 365 targets/96 batches，195 命令/390 raw，合计 7130/0/196、730 suites；逐 target 的 metadata/artifact/Running/named outcomes、4886 来源及全部 required gates 独立接受。额外六条 ignored 命令共九测试、两个公开示例及 feature/normal/layered dependency 检查通过。

workspace-v9 的计费失败保留。原 recorder 重测六 case，仅 many-method-class 增六 IrItems；旧 direct/shared 两 arm 的真实记录也各增六，其余维度不动。三个精确 pins 按新预算化扫描更新，保留全部原 shape/text/outcome 比较。原 v15 类型编译错误、v16 故障注入未去除 Integer 固有类型、审计依赖准备错误和正向审计器对负例 content 的不适用要求也分别保留，不拼成成功。

本地运行采用默认 Rust stack、5 GiB free/target 1 GiB/一秒进程组守卫及 finally exact cargo clean；最终 target absent。本地 Java25.0.4.1 runtime/JDK23 compiler 组合不冒充 GitHub 完整 Temurin 工具链。旧 f014 自身 CI 虽为 4 jobs/52 steps 成功，已知语义反例使它不能关闭本次交付。

[capture.tar.gz](capture.tar.gz) 为 23673727 bytes，SHA `c5ba3c8f273493df9cdcce092fd7de49a83df77b37b47678797552c022c8d95f`。root 独立复开 3536 payload，十个 final-source 与 focused/workspace 编译来源及当前文件逐字一致；4886 完整来源闭包相等，见 [archive-review.json](archive-review.json)。大 JDK/JADX/CLI 安装内容未打包，工具及可执行文件身份保留 pins。

这里只接受最终本地证据。修复产品提交、它自身的实际 CI 与最终 clean-main 仍待完成；不增加 CF11/CF18/71/612 总体完成计数。
