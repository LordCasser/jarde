# 052：计入两FO fixture后的新构建与完整源码重放

产品052c82484bb20ca8b7e22ffe9943503c961280f9已推送，只有reader cfg(test)人口元组和四行说明改变。独审在内存逆掉这项变更即可精确重建a89原classfile.rs字节，decoder及其它断言未改；两FO class独立解码恰新增2类/10 Code/2 handler，无branch/jsr。新reader原具名人口test实际1通过，未以删样本、放宽解析或改P5计费表通过。

新native实际6命令全部exit0：fmt、reader exact1/0/0、正常FP5/0/1、P5 bulk all-features5/0/1、CLI构建、exact clean。共11通过/0失败/2默认忽略。完整4915源码before/after相同，并由实际052 selected-path ls-tree逐项核Git blob/mode；target缺失。新冻结CLI59,699,632B/0555/SHA256 6e075292bf761f178858f5714b3dbbf51e57a9e711ca1d87d45e77312228db96，与a89旧CLI逐字相同。

本包仍另做了真实新14命令完整源码重放，并非重标旧a89收据：两个完整五方法report.text保持1617B及SHA eabbf22187e0b3c24bb984d9496a4785acc479fe05ca9e07acf8952337e25811，分别经真实javac8和javac23 --release8原样重编；未改driver仅对rebuilt编译，四main和四probe在Java8/23的隔离rebuilt-only classpath精确匹配6B/60B oracle。没有改生成Java或删方法；原类不混入。源码/输入前后相同、默认stack与5GiB/1GiB磁盘门、exact target边界均核。

独审198检查/0错误，接受的是新本地6+14共20命令/40份command raw。a89祖先完整本地Clippy/MSRV/strict检查另见dispatch-local-v1，不改称本次052执行；新052自身CI38859590941是独立终态门，未在本包接受。此前c075计费失败和a89人口失败均保留。能力/owner/停止阶段边界仍按design，不增全局覆盖数。

capture.tar.gz：2,528,942B、SHA256 52e0d559703c5a52742703a4a8cd98edecdffee9216c9d7b612a376843f6d9fb；84 payload加manifest共85安全唯一普通成员，root已重开全hash。CLI/JDK为外pin。
