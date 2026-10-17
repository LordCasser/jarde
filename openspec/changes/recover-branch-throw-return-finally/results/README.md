# finThrow 实际证据索引

所有 capture 为原始执行与输入/生成源码的归档；root-review 描述该次实际接受范围。不能把私有草稿、历史 CLI 或失败工作区拼接成最终通过。

- baseline-root-v1：原 class、原 JVM 行为、实际首拒探针；旧 CLI 的 cfg(test) 探针来源单独说明。
- baseline-matrix-root-v1：旧产品完整生成源码红基线、固定 JADX1.5.6 的实际重复 cleanup 错码。
- focused-and-boundaries-root-v1：最小入口修复后的公库/guard 与真实负变体；早期错误 CP/row 边界准备失败也保留。
- local-final-gates-root-v1：450 库单测、2 API、指纹再生/核验、reader census、fmt/CI同款Clippy、冻结 CLI。
- final-matrix-root-v1：4 profile、8次完整源码原样重编、16次 rebuilt-only JVM运行、144 case；原 EX 六方法保留，nestedFin 尚红。
- workspace-old-negative-failure-root-v1：首轮工作区在旧 anyFinally unsupported 预期处真实失败；不是成功收据。
- precise-neighbor-gates-root-v1：仅更新上述测试预期后，2 precise、19 P5、fmt/Clippy通过；重建 CLI 与最终矩阵逐字相同，明确复用该矩阵而非声称重跑。
- strict-all-root-v1：365 项严格校验通过。

- workspace-seed1-root-v2：完整375targets/98batches、3686/0/98、4927来源、202raw；独立验收按实际Running顺序重建每target结果，保留历史recorder编译顺序标签但不把它当执行顺序。

- workspace-two-seeds-root-v2：双seed完整验收，各375targets/3686通过/0失败/98忽略，合计7372/0/196、404raw、相同4927来源。

- own-product-ci-root-v1：精确产品6b32自身CI39037102727，4jobs/52steps；两seed各375targets/3686通过/0失败/98忽略，root独立接受。

finThrow当前5/5，生产/测试源码与已验收产品一致；关闭文档提交后另核实际clean main。父 finally 仍4/7，71/612计数不增加。
