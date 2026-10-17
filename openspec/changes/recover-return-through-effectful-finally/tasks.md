# Tasks

## 当前状态

最终组合产品为 `4c6852ac8c4ab9ce88f0716f4d2bd702e15684c5`，冻结 CLI SHA256 `4811eea2d7f52d4c1b818c86acc1d961c91e3d7fe9d5adaa2c53131f84140f8e`。finThrow已5/5，nestedFin完整源码/行为、负边界和自身CI均已验收；两个本地workspace seed及父文档双版本strict均已闭合。父级当前7/7；30afa/05e、原patrol与曾收窄范围的7/7均保留为历史。

## 1. 取证与基线

- [x] 1.1 复跑patrol完整FY/FX/FO并单列当前矩阵，不改历史README：最终24命令保留两编译器来源6完整类/32方法，全Java/Structured无fallback，6次原样重编0与12次rebuilt-only main逐字等原oracle；见leaf final-runtime-v1及独审。
- [x] 1.2 补齐instrumentation.md：历史Q-i对应loopless实际首拒16!=25与融合尾48/49，Q-ii分别对应noCall真实iinc/handler phi与CallReturn保存调用结果/cleanup异常链，区分transfer不可达handler与可抛调用；逐项引用已归档事实，登记新finally返回覆盖/主动抛异常红项。

## 2. 实现

- [x] 2.1 补齐单return穿副作用finally及已观察的其它非覆盖型清理：既有五叶片和新增finThrow/nestedFin均已按实际Facts/SSA/effects/异常边实现并验收，复用Finally/SavedReturn，判据不放宽。产品4c6852的同一最终CLI完成nested432及finThrow144个rebuilt case，完整EX六方法保留、4次原样重编成功、8次nested driver运行通过，EX.finReturn仍fallback且文本与旧基线相同。原失败raw保留；FinallyCompletion七方法完整保留，8次整类编译仍各三处missing return。tryThrowsFinallyRuns是后续独立正向欠账，finally自身非局部完成继续拒绝；不能把它们称已恢复。证据见nested叶片final-matrix-root-v1、finthrow-regression-final-cli-root-v1、finally-completion-refusal-controls-root-v1。
- [x] 2.2 FO求值序判别测试入测试集（1/100）：recover_static_field_prefix_return三项永久测试与已归档完整源码/driver证明先保存再finally；历史target为3 passed，最终父矩阵再核FO两编译器/双JVM固定main。后续若测试入口改变，必须核实际selector覆盖这三项。
- [x] 2.3 完成对照验收独审：emptyFin及28个非transfer方法文本相同，continue/break按授权去fallback并以原oracle/完整来源约束，未来固定30afa为完整基线；EX.finReturn相同fallback和FinallyCompletion三处missing return按具体形保留红项，不冒称健康finally-return对照。

## 3. 验证

- [x] 3.1 在2.1正向形完成后汇总并独审FY/FX全锚、FO判别及EX两相邻形独立完整fixture：nested叶片parent-fixed-matrix-root-v1实际24条Jarde命令复验6完整类/32方法，全部物理BCI/映射/身份接受，32文本同原基线，6次整类原样编译及12次rebuilt-only main等固定原oracle；nested与finThrow完整矩阵及独立driver边界见各自root-review。JADX21命令19成功/2次FX真实编译失败、0 FX运行；nested的6编译成功/12运行IOIO首错保留，不修外部文本制造三方一致。
- [x] 3.2 最终生产实现后完成父源码/CLI/CI闭合：4c6852同源的29本地门131 passing appearances/127不同FQN，MSRV/fmt/CI同参数Clippy/reader/P5/双strict均接受；native两seed及该产品自身CI39042611463两seed各376targets/3691通过/0失败/98忽略。父计划更新后再次实际运行OpenSpec1.11.0与1.14.0严格校验均365/0，root核完整范围与收据；后续文档head仅按相同生产源码绑定，不冒称自身CI。实现验收7/7，尚未同步/移动归档。

## 阶段验收与范围纠正

固定FY/FX/FO的2.3对照已独审通过；旧30afa证据不替代最终4c6852门禁。2.1曾被收窄到三锚、短暂7/7，root重审后恢复待办；现在finThrow/nestedFin已实际完成，2.1/3.1依据新证据重新关闭，现最终3.2也按同一产品的两个本地workspace seed、自身CI及父计划更新后双strict365/0重新关闭，当前7/7。历史收据与纠正说明保留，不伪造归档。JADX FX仍编译失败，finally自身return/throw仍是拒绝边界。
