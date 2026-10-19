# 定向门、完整原件与可达控制例验收

root 限域接受本片定向和完整对照。归档 576 个 payload、17646820 bytes；capture.tar.gz SHA-256 `895e058e6e2114c21f42d7634b95c8f1eae71e21c1608f8aad5cc1e26552f89b`。成员 bytes/mode/hash 已复开，清单见 inventory.json。产品 native/自身 CI 另行验收，不从本记录推导整体交付。

最终 CLI 为 64,347,312 bytes、0555、SHA-256 `7e321a7f4fd6a029540ab169f79572c6ad83e74119900ac6d695290c9c055318`。final-focused v2 与 first-focused CLI 完全同字节，完整矩阵可精确复用；二进制不入 Git，构建收据保留来源快照。

定向门：Java lib 526/0、公开入口2/0、Reader1/0、指纹5/0/1既有忽略、fmt与全CI Clippy通过，target峰值682,969,441 bytes；source零变化、guard无事件、finally清理。Reader总量1351 class/5945 Code/646 handler/4312 target/16 jsr，与新增10 class的独立raw-Code统计一致。

原件完整对照：94命令/188 raw，八API（class/普通JAR × debug/no-debug × default/all）全部保留四个方法并结构化，来源17/17 BCI、default/all全文/identity/quality/maps一致。16份Jarde完整目标类在JDK8/23均重编成功，32隔离VM ×2输入=64观察同原件；JADX完整outer树JDK23两重编/四VM八观察同原件，两个JDK8原framework55/52边界保留。18个生成错根检查全拒绝，另保留原件错根控制。源码、生成完整类及原流全量保留，未裁方法或手修生成Java。

原件funcB恒正常返回，catch/clear/sleep不可达，不能把上述正常路径观察算作catch动态覆盖。另有独立控制例：四编译腿/双VM40原件观察；候选八API、16整类编译、32VM160观察与原件一致，覆盖正常、失败后重试、同异常实例重抛、重试效果异常逃逸及条件反向。16份生成class经两个解析器核对类标志、方法/字段库存及访问标志无差异；不声称所有JVM属性完全相同。16个候选错根控制全拒绝，driver目录仅含Runner，目标目录仅含完整生成类，无原helper二进制注入。

四个真实拒绝邻例经root独占的JDK8/23 -Xverify:all加载与方法枚举验证，然后经Reader/CFG/SSA及候选拒绝/所有权测试；修改的方法没有动态执行，不当行为等价证据。

失败与审回材料保留：控制v1超出有限分支且漏接异常、v2导入错误，v3后才实际运行；Luna实现v1移动后使用及重复扫描收费经root v2修正，旧单块预算保持；final-focused v1是负例错接正例helper，指纹再生成是唯一预期源码变化，修正测试helper后v2全过。另一个Luna违反“不得运行JDK”的派发边界，其不完整自报不算验收，root重新准备并验证四个输入，相关历史原样归档。

独立完整矩阵复核位于归档 independent-matrix-review/audit.json，SHA-256 `ee2a396aba4420e47adb166df27d293edfd4b794eb7c91d3b967e20179c559ef`，无发现；原件专用reviewer另复开13,313个pin。观察只覆盖明确列出的输入，不提升生产verification状态或宣称普遍等价。
