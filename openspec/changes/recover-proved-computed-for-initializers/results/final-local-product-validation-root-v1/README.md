# 当前产品本地完整验收

root独立接受fresh workspace v2：95批、359个metadata目标和359份结果，3451 passed/0 failed/97 ignored，8零测试目标按实际artifact核对。4700源码/包含/fixture闭包恒同，峰值811131401bytes、最低free19350835200bytes；已删除完成测试可执行文件并最终cargo clean，无target。

最终fmt与同CI29项Clippy允许表的全workspace/all-targets/all-features/locked -D warnings三命令通过，root独立核CI argv一致并clean。OpenSpec strict349/0。最终CLI build+clean两命令真实通过，0555/60821472bytes/SHA d1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1，完整4700输入同workspace/live/冻结closure。

最终CLI与已执行对照的候选CLI逐字节相同；root verifier绑定四份实际接受receipt（正常对照与共享输入可靠拒绝分别记录），未宣称重新执行146条对照命令。二进制保留receipt私有冻结位置，不复制60MB二进制。

源码产品尚待提交推送及自身CI核验；本目录不冒称Git/CI或clean main已交付。原raw/receipt路径字节保留，inventory绑定复制。
