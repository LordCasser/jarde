# 当前循环体 catch 基线（观察验收）

产品提交 `ca45c0ed2193fa8709eeba1196ef325f1804f63e`，CLI v4 SHA `d3d15d3dd875bc79410298e3ed1e56010fc39c1849e97dc893cadb49d8c65e6f`，focused v12 SHA `74350dfc154c2fd77aa8a9636dd8d29b301eb119e167be56a0f18af011996a73`。root 串行执行带/不带调试信息 × Java8/23 两份基线及默认包名补充，再独立检查112份原始流、工具/输入前后摘要和56条命令。这里只接受观察，没有实现或语法单元关闭。

原类四腿的18个输出逐字节一致。JADX 无重命名四腿与默认重命名四腿均可整类编译运行，但 `DF.loopCatch([1,-2,3])` 均抛出 IllegalArgumentException，原类返回3。默认配置先因生成 defpackage 与原 Runner 命名空间不同而编译失败，该原始失败完整保留；补充只给观察用 Runner 添加精确 `package defpackage;` 前缀，未修改反编译的 DF/DV，不把适配失败归为产品失败。

Jarde 四腿完整类均编译失败，没有运行。DV.plainFor/whileLoop 与 DF.loopCatch 为不可归约拒绝；DV.forEach 已结构化循环，但随后 local 2 的跨 protected-region 定义证明拒绝，正文为 explanation_only。其首个 enumswitch 警告不是阻断恢复的诊断。DF.loopFinally、无 try 及 default 形保存为当前对照，后续不能承诺它们已完全恢复或用本次观察证明零回退。

原 v3 collector preflight 的 Runner 路径失败（commands=[]）未改写。root 初版观察校验器错误假定 Java8 javap 也打印 Java23 SHA-256 行；v2 改用 original.classes 的完整 class pin 验证，再交叉核 javap 原始输出。没有重跑产品工具链来掩盖校验器错误。

完整原/JADX/Jarde 源码、class、异常表、命令、raw、同物理方法身份和source maps都在归档。Q-i/Q-ii/Q-iii 的同次 IR 证明仍待实际采集；没有从上述拒绝文本猜测门内原因。
