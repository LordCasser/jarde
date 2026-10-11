## 1. Diagnosis and bounded implementation

- [x] 1.1 固定真实 class/JVM/Reader 首个拒绝点及架构审查。验收：SHA 与 lib-v9 raw 诊断一致，明确普通内部块被直接入边校验误拒，独立审计记录覆盖、不达性和预算边界。
- [x] 1.2 Luna 私有实现、root 审查后应用最小 assembly 覆盖修正。验收：复用现有邻接，无新增公开结构/pass，ordinary 图无费用变化，克隆 rooted 路径闭包及新增槽/工作收费可逐项核对。

## 2. Adversarial verification and delivery

- [x] 2.1 真实完整 class 经 Reader 取得 CanonicalCFG、Frame、SSA，并保留两套克隆来源。验收：双 JDK 原 class 合法性证据独立，root 实际运行新 Reader/region 回归，拒绝原因不能仍停在前端。
- [x] 2.2 加入断开、自循环、错路径和未覆盖返回负例，回归共享/嵌套/handler/fusion/死调用场景。验收：root 实际 jarde-jvm 全库通过，mutation 明确仅覆盖规范化发布层，unreachable 真值保持。
- [x] 2.3 新增覆盖存储/工作预算与取消原子停止。验收：root 实际观察正确 dimension、usage、停止传播，无 CanonicalCFG 与后继 Frame/SSA 半产物，历史取消阶段通过。
- [x] 2.4 与父片最终源码完成格式化、适用测试/lint、OpenSpec strict、同 CLI 原/JADX/Jarde 对照及自身产品 CI，更新 verification/handoff 后提交推送清理。验收：真实命令/raw/source pins 可追溯、main 干净且不占功能分支；不宣称完整 verifier 或 Java 源码覆盖扩大。
