# 三方运行输出 SHA-256（java -Xverify:all；JDK 23.0.1）

| fixture | 原 class | 固定 JADX (dev) | Jarde 重编 | 判定 |
| --- | --- | --- | --- | --- |
| I1（fam.jar，四路径含 `hello:v`） | 40a24bddb0db3dad7e2f273b2d4c43e27daa6b7b8ab5913941c54e3a8f7fc594 | 40a24bdd…（=左） | 40a24bdd…（=左） | 逐字一致 |
| H1（hier-fam.jar，8 变体路径） | f4bdad2c35ae4f3bc6f02934f340a5923235f8f659395a5a04a6a55b58922c70 | f4bdad2c…（=左） | f4bdad2c…（=左） | 逐字一致 |
| H2（hier-neg.jar，负例家族） | 80a211c34fd92649c9989c94164091ce5b3efc4a1db86be404a19a394cbc7bfd | 80a211c3…（=左） | 不编译（恢复文本含 2 处拒绝，负例即验收） | 原文/JADX 一致 |
| H3（hier-depth.jar，8/9 边深度界） | 842e854d8d8baee0d3d29aa237a4ce92c34189f8c6c16719404244b45371645c | 842e854d…（=左） | 不编译（第 9 边拒绝即验收；第 8 边 L7 位恢复） | 原文/JADX 一致 |

JADX 列说明：jar 输入被 JADX 置于 `package defpackage;`；I1/H1/H3 以 `defpackage.X` 运行。H2 的 `H2$Ext extends Helper`（Helper 在默认包）按 Java 语言规则不能被命名包引用，故去除 `package` 行后与 `fixture/hier/Helper.java` 同编于默认包运行（字节形状与原物一致：默认包 H2 extends 默认包 Helper）。Jarde 侧 H2/H3 不做重编运行：其恢复文本分别含 2/1 处拒绝位（负例与深度界即验收对象）；原件与 JADX 两列已三方一致。复现编译：`javac --release 8 -g:none fixture/hier/{H1,H2,H3,Helper}.java`。
