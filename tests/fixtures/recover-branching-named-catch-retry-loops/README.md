# 条件重抛与异常重试循环

原件 source/TestBreakInLoop2.java 完整保留外层 JUnit/check。original/debug、no-debug 的目标 class 与已验收完整基线逐 bytes/SHA 一致；它的 funcB 恒正常返回，所以 catch 行为不能由原件运行宣称覆盖。

source/RetryCatchControl.java 与独立 Runner 是可达控制例，control/ 下保存 javac8 与 javac23 --release 8 的 debug/no-debug 完整目标 class。两个 run 方法分别以正/反条件选择重抛，throw 叶或单 retryEffect 叶，效果自身异常不受原 catch 保护；没有真实 sleep。root 已完成原 class 四编译腿、双 VM 共40观察、四错根拒绝，候选恢复/重编对照仍待完成。

实现/验收入口：openspec/changes/recover-branching-named-catch-retry-loops。拒绝的控制例v1/v2和Luna/根候选均留私有历史，不能当已执行证据。

rejections/ 的四个邻例基于原debug class：BCI31 goto改34；BCI6 goto改19并将19/34 StackMap换为合并后的初始locals；17/18改nop/return；复制唯一异常表行并调整Code属性长度。四者已由root独占在JDK8/23 -Xverify:all中仅加载并枚举methods验证，未执行被修改方法（其中可形成无限循环）。它们用于实际Reader/CFG/SSA拒绝测试，不是语义等价候选。JADX源码及派生class的许可保留在licenses/。
