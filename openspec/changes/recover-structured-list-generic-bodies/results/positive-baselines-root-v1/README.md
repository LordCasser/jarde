# 不同类名、元素型的完整类基线

root 冻结 `DistinctStringFixture` 和 `DistinctIntegerFixture` 两个合法完整类、各自 Runner，实际按 debug 与 `-g:none` 两套输入执行双 JDK 原/JADX/旧 Jarde 对照。每份 capture 29 条命令，共 116 条；各份 collector 与 root 独立 checker 均实际 exit 0。没有编辑反编译正文，没有截取方法去绕开整类错误。

String 的 5 个输入包括空列表、add 后 CME、对象 remove 后 CME、null 元素、null 列表；Integer 的 6 个输入另含合法 raw 写入 String 后的 heap pollution。每行保存异常类、两个集合状态和实际 mutation effects。String 原始 stdout SHA-256 `ca4dd6cf6e305d7914bd7f6fbd9d94c9c3af74000c84bf76ca31c0089d2c2cb6`，Integer 为 `0b5c5b91fbda5ade2704697a3df5818d7884f51914df886492a0f32c374f6329`；每家族 debug/none、Java 8/23 四条原类运行逐字一致。

每个家族每套输入的 4 次 JADX 和 4 次旧 Jarde 完整生成类编译、运行均 exit 0，stdout/stderr 同各自原类；两家族合计 32 次生成类运行。Jarde default/all 的完整类、方法与原始 recovery maps 一致，所核原始 maps 没有物理 BCI 缺口。旧 Jarde 仍输出 raw List，行为一致不能替代泛型声明恢复；当前新生产候选尚未应用或接受。observer 仍声明 `behavioral_acceptance=not_claimed`，class BLAKE3 仅核形状和一致性，未独立重算。

`private-fixture-prep-v2/` 保留 Luna 源码构造及内容验证输入。两个独立类名、参数名、元素型与固定 TestNestedLoops 区分；源方法从已冻结合法源码确定性提取，仅在成功 add/remove/clear 后加入 effects 计数，运行中实际验证，未把修改反编译源码当作成功。合并控制的旧局部生命期/handler 缺陷另见相邻记录和独立债务账本，不因这些正向类通过而关闭。
