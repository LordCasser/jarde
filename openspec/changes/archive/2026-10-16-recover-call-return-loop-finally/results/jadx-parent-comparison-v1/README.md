# 固定JADX父级完整源码对照

capture.tar.gz 为4,137,847B，SHA-256 486108895130e0b10f6eee40f104c8b9db11c28ebb8fd1ee27ae5af7ec40b3bf；105 payload加嵌入manifest共106安全唯一普通成员，已逐项hash重开。独审96acd7a4689b27000a29e402fc84472a814e02d7b77c536d09e7eeb6a177e4d9。

实际JADX1.5.6输出完整源码，v3为21命令：19 exit0/2 exit1。FY/FO两编译器4完整类原样重编成功、8次Java8/23 main与保存原oracle逐字匹配；FX两份完整输出均在throw th处因th未定义编译失败，无FX rebuilt runtime，不修文本/删方法。JADX把原默认包输出为defpackage，运行采用实际生成的defpackage.FY/FO，并明确记录包变化，不能声称原物理类身份未变。

v1用java -jar进入GUI导致Headless、2命令后无源码；v2改正确CLI入口但未采用实际包名，4命令后FY运行找不到类。两次harness失败原流保留，不并入成功计数。v3以-cp ... jadx.cli.JadxCLI执行，生成源码保持原样。Jarde最终parent screen另6完整类/12main匹配，但finContinue/finBreak仍handler45 fallback；两工具结果都不代表异常handler或父全部anchors完成。工具及CLI/JDK二进制仅pin不打包，包内绝对路径按manifest映射。
