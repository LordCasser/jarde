# 循环内局部汇合与直接 continue

当前2/6，只有原始事实与架构边界已接受：[完整原始基线](../../../evidence/java-syntax-2026-10-19/cf09-testcontinueinloop-original-root-v1/README.md)、[同轮IR及首次内部gap](same-run-ir-root-v1/README.md)。Luna私有实现/fixtures与公开测试/最终对照脚本正在并行准备，没有生产代码集成或候选验收；当前产品CLI仍为上一布尔片SHA9ea9177b8992c8444c9f9af0158ca7bc99b3e7dfee4696b342aea7da2e295167。

root先完成上一片cc1bf产品自身CI采集闭合，再审私有差异并集成。共享块42的重复占有是原始实际失败，不是放松覆盖门的理由；ForHeader已成功，修复应复用既有局部join/LoopContinue及有界完整性证明。CF09/71/612仍为部分覆盖。
