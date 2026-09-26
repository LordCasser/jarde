# 2.3b 子类成员与正文证明验收

2026-09-26，root 审查 `src/facade.rs` 的选定物理子类、成员表、typed Code 与 class-source 恢复路径，并独立执行 `CARGO_TARGET_DIR=/tmp/jarde-enum-root-accept-target cargo test -p jarde --lib enum_constant_body_relation_tests`：9/9 通过。代理执行完整 `cargo test -p jarde --lib`：68/68 通过；格式和 diff 检查通过。严格 Clippy 在未修改的 `jarde-java` 既有警告处中断，未对本步骤形成通过结论。

证明只针对 2.1b 选定的物理 definition，沿同一 Budget 恢复子类 class-source；typed Code 再核对每个潜在覆写的异常表。成员表必须只有唯一无额外效果的构造器与完整、可拼写的实例覆写；字段、`<clinit>`、非法可见性、无唯一主类声明、异常体、fallback 或来源/声明不全均拒绝。class-source 覆盖记录核对每个 Code 成员在该次运行中恢复一次；all/essential 公共证据选择不改变内部证明。

正例 `Op`/`Mixed` 的方法身份与文本可复核；字段、构造副作用、非法覆写、异常表与紧预算控制均保持 `Refused` 或 `Stopped`，没有半成品常量体。此步只保存私有 `body_proof`；2.3c 整组合证与 3.x 类级投影尚未实施。
