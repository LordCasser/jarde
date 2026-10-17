# 补充完整类原始输入

两类原样源码及8class来自root接受的`results/supplement-originals-api-root-v2`，尚未代表恢复实现通过。每类8方法全部保留；javac8 source/target8和javac23 --release8各编debug/no-debug，双JVM16运行共160原始case同各自10行oracle。

源码和driver在source/，class按编译器/profile分组；copy-manifest固定bytes/SHA及原始编译命令索引。source-manifest保留Luna源码交付时“未编译”状态，之后的真实编译/运行以OpenSpec证据收据为准。

两个execute的控制骨架与原形相同，但非空catch使javac插入纯outer latch bridge：protected64的goto@69→94，再由goto@94→outer18；handler72的goto@91→inner36。所有位置来自本冻结Code，不作为产品匹配条件。测试覆盖前缀效果、try正常完成、catch后再次执行内层前缀、非局部条件退出及异常逃逸；catch后stage gate会退出outer，不会再次执行protected调用。
