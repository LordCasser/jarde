# Receiver-tail 计费债务

root复核本片对抗审查及当前 `init.rs:622–634`、`receiver_tails` helper 后，决定单独登记，不混入构造参数数值转换实现。

本片已将普通 allocation census 指令扫描、普通 construction verifier 和参数依赖 walk 接入同一次 request Budget，并将 Stop 原子传给 report。收尾的既有 receiver-tail 展平、排序、扫描以及计划合并/排序没有因此变为逐项计费；不能宣称整个 Site census 全部实际工作都已计费。既有 helper 不受 PrimitiveConversion marker 控制，也未被本片新增或扩大，因此这项债务不影响当前转换、handler 或半计划发布结论。

后续独立片先核查 receiver-tail 的全部生产调用方及现有预算口径，再复用现有 Budget/Stop 接口计费，不增加规则开关或长度估价。至少冻结一个真实 dynamic bound-receiver 正例，在收尾中实际耗尽预算或取消，验证停止报告不发表正文/records；若扩至 owned 合并和排序计费，须明确口径并按实测重录 P5 pins，不将计费变化写成性能提升。当前 ordinary focused 输入的 pending map 为空，不能据此宣称测试直接覆盖了非空 pending 移交后 Stop 的组合。

本文件仅记录已见债务与后续验收范围，没有改动产品、测试或当前 slice 的功能边界。
