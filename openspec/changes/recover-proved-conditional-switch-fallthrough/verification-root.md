# Root verification — recover-proved-conditional-switch-fallthrough

本片当前 **1/7**；条件switch生产候选仍在private目录，尚未应用或编译。完整证书helper、SwitchBreak叶及最近switch作用域的root静态审阅已完成，但不替代真实helper/caller、预算、完整源码和自身CI验收。先关闭局部类型片的准确产品CI与clean主线，再应用本片。

## 两个完整物理负例的实际观察

[physical-boundaries-root-v1](results/physical-boundaries-root-v1/README.md) 保留原class完整副本及两个offset-only变体，两个JDK各original/A/B共六runtime腿，每腿36行，16条实际JDK命令全部通过。root审阅私稿后纠正四字节/五member的错误期待；每变体实际只改两个低operand字节，完整构造器与五业务方法保留，无匹配变体的Java源码。

[独立public IR接受](results/physical-boundaries-root-v1/public-ir-acceptance-root-v2.json) 只接受观察：四partialBreak profiles均6块/8条Normal完整multiset、empty clone paths、完整incoming与SSA membership；A case0 exits57/67，B只有67且57在物理入口次序中间。物理50/51/53/56不在canonical区间，canonical.unreachable原报告为[]，两事实分别保留。实际BLAKE3/owner/snapshot/name/descriptor、decode37/47、raw和86source pins已核。临时observer移除，守卫峰值180560145bytes，cargo clean348files/172.2MiB，target不存在。

helper/caller gate与新产品尚未接受；private tests v1/v2均未编译，不能当任务完成。A须实际到达多出口拒绝，B须分开证明helper的唯一map与真实caller的adjacency拒绝；不复制生产门逻辑作为测试。先前完整五边界原/JADX/Jarde观察及十个原class public-IR profiles分别保留在boundary-baseline-root-v1和boundary-public-ir-root-v1，新负例不改变那些历史结论。
