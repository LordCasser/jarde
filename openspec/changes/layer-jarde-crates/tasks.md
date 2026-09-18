## 1. Baseline and reader

- [x] 1.1 确认 P2 `0.3/0.3b/3.4` 及其依赖已验收，保存精确基线、测试结果和生产依赖树；按 design 盘点跨包私有访问及测试辅助消费者，清单中逐项注明所有者与最小公开面，未绿不搬迁（2026-09-18 **分析部分完成**：基线 `0f3134b`、628 passed / 0 failed / 1 ignored、生产依赖树与 19 个待拆模块已记入 verification 的 1.1；盘点 185 个 `pub(crate)` 项（140 个跨文件引用），按「接缝 / jvm 内部 / 无需公开」分类，产出 10 条接缝归属表、7 项**搬迁前必须处理**的编译失败项、门面必须迁入 jvm 的清单，见 1.2；三处 design 未定项已定案（blake3 收敛进 reader、`CandidateFilter` 不整体公开、`read_entry_internal` 改名，见 design §3.5）。**未做**：7 项前置动作的实施与文件搬迁）
- [ ] 1.2 抽出 `jarde-reader` 及检查入口，将共用 budget/error/model/view 与唯一解码留在该包，提供有界物化和只读 facts 接缝；验证 reader 独立 check/test、快照/身份/字符串/操作数/精确预算边界与现有 P0 oracle、P1 golden 不变，确认无向 query/jvm/facade 的依赖

## 2. Query and JVM ownership

- [ ] 2.1 抽出 `jarde-query` 的 query+xref 与最小 candidate scan 接口，同步 resolver 调用而不改变其语义；运行只依赖 query 的实际查询 consumer，独立 check/test、分页/coverage/cancel 回归及声明引用对照通过，生产和测试依赖均无 jvm/facade/petgraph
- [ ] 2.2 抽出 `jarde-jvm` 的运行环境、resolver、CFG/call-context/pass/IR 及方法分析 driver；门面只保留委托与选定再导出，用当前全部 P2 反例验证阶段/预算/身份/原 BCI 一致，不为搬迁公开 HeaderClosure、FactLedger 等可变内部结构

## 3. Integration and gates

- [ ] 3.1 同步根包/CLI/examples、跨包单元测试辅助、fixtures 路径、fuzz path/lock 和 CI 包选择；第三方版本/features 不升级，验证库/CLI 同请求逐字段一致（沿用 elapsed_millis 归一）、fuzz targets 可构建且固定 replay 通过，不复制生产 decoder 或新增空 java/common crate
- [ ] 3.2 更新 A17 的源码路径守卫并增加 Cargo 依赖闭包门禁；以临时加入 query→jvm 依赖、向 query 注入图分析调用的反例证明门禁会失败，同时保留 X0/X1 不启动分析、单方法不读无关 Body 的行为侧证据（A14/A16/A17）
- [ ] 3.3 完成 workspace fmt/clippy（-D warnings）/全量测试、P0 oracle/P1 golden/P2 回归、MSRV、两套 workspace supply-chain、既有 fuzz 门槛与 OpenSpec strict/diff 检查；记录实际命令、精确 commit/CI 和只读复核，更新架构/依赖/阶段源码路径后交接 P2 3.5，不将未执行门槛记为通过
