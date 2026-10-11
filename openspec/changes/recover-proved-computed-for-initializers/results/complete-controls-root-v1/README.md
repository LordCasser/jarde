# 完整类原/JADX/Jarde 当前基线

root fresh controls-v2共59命令，原class法律/九行行为见controls-baseline20命令。JADX十二完整编译/验证运行成功，其中普通javac controls四组前四行与原版不同（f/g初始化被跨独立调用提升）；pending四组、direct-return四组全部同oracle。Jarde复杂controls八次完整编译因fallback缺返回而失败，没有runtime；direct-return四组完整编译/运行与原四行相同，仍While。default/all完整正文/map/质量恒同，独立root实际验收全部argv/raw/工具/63jar/输入/生成class与Runner包装。v1的47命令失败为JADX默认包defpackage与Runner包不匹配；v2仅前置package到相同Runner正文，JADX完整源码原字节不变，不算修复算法。

冻结CF07 LoopCases另fresh JADX六命令，两JDK完整源码+固定Runner验证运行均与当前原/Jarde基线八行相同。输入419字节SHA004e99…32ae33，单成员jar精确对应该class，launcher1.5.6，原/currentJarde见planning-root-v1当前双JDK对照。本批只接受准确基线/错误与拒绝，computed-for实现和71单元没有完成声明。
