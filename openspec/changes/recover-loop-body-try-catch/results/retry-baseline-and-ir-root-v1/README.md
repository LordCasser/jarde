# 无限重试循环：当前基线和同次 IR

root 实际执行并独立接受两组完整类基线：v2 保留动态观察文本，v3 仅将事件替换为固定文字并移除 AssertionError 消息。每组 debug/none × javac8/23 共44命令；原 class 四次运行与 JADX default/none 八次运行均同五个计数/事件 oracle。每组 Jarde 四条完整类输出编译失败，无 runtime；没有把失败删掉后拼接为成功。

v2 的 exc()V 另有 nested StringBuilder 构造参数 jre_new_shape；v3 消除该观察干扰后 exc()V 无非 info 诊断，test()I 的首拒仍为 BCI0 region ownership overlap。该构造差距独立记录，不扩大本 change。

同次 IR root 实际8测试各1/0/0，加最终 exact clean，共9命令/18raw；root 独立 checker 实际 exit0，完整4822源码恢复且 target absent。每个物理 test()I 与同 lane CLI 的 class digest/length/name/descriptor 配对，Code/canonical/normal-flow/SSA 和 recovery 全部 Complete。普通图无环；handler 普通跳回0，异常边0进入handler。所有 lane 都是同一个根 Try 的 catch 再进入嵌套同 Try，使 block0/handler 各有 owners[0,0]。证明的是当前拒绝位置，尚未实现恢复。

插桩只用最小 MethodFacts（参数槽与访问标志），不是 CLI 的完整 debug-name RecoveryFacts。源码临时变化仅 region.rs 的 cfg(test) 观察和私有测试，已全部还原。

capture-and-tooling.tar.gz 保存两个 baseline/prep、194个raw流、物理 class/javap/生成源码、baseline与IR独立checker/收据、IR prep/runner/原region源码以及逐文件SHA清单。CLI/JAR仅保存其收据身份，不复制二进制。root 已重新打开包逐字节核对全部文件。见 summary.json 中完整 root 接受结果和 archive SHA。观察接受不关闭产品任务或提高71/612完成数。
