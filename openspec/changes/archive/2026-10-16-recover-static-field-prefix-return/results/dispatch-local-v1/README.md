# 后继 a89：新 CLI 的完整源码与本地产品门

产品 a89d423c9a6e5d62ceb3f8185a818ebe3d67a17b，仅给静态prefix规则增加固定六opcode候选筛选。真正候选的计费/SSA证明原字节保持；旧P5计费表和断言未改。新 CLI 59,699,632B、0555、SHA256 6e075292bf761f178858f5714b3dbbf51e57a9e711ca1d87d45e77312228db96。三组实际执行的4915源码 before/after 完全一致，逐项Git blob/mode绑定a89原树。

实际30命令、60份command raw：新native6命令12通过/0失败/1忽略；新完整生成源码14命令全部exit0；新本地产品检查10命令全部exit0。两原编译腿全部5方法report.text原样1617B（SHA256 eabbf22187e0b3c24bb984d9496a4785acc479fe05ca9e07acf8952337e25811）分别用真实javac8及javac23 --release8重编，未改driver仅对rebuilt编译，四main精确1/100与四probe精确60B六行，classpath不混原class/JAR。没有改生成Java、删方法或借原类运行。

本地检查含fmt、CI原参数Clippy、fingerprint5/0/1、P5 bulk all-features5/0/1（原两计费断言具名ok）、实例更新3/0/0、静态postfix旧值4/0/1、Rust1.88 workspace all-targets，以及OpenSpec1.11/1.14 strict各364/0。四套合17通过/0失败/3默认忽略；ignored不算执行。默认stack、5GiB free/1GiB target磁盘门与exact clean保留；target缺失，峰值615831294B。这里只接受本地门，不称已运行本地完整workspace双seed。

独审native/完整源码179检查0错误，本地75检查0错误。c075祖先本地成功与自身CI计费失败分别保留；新head自身CI是独立门，须另见own-ci结果，不能借本包代替。owner保守子集、动态停止实测预算/取消而非deadline、正文提交与后置可选证据阶段的边界沿design保持，不增全局覆盖数字。

capture.tar.gz：3,521,348B，SHA256 1f2683638eba64c39b98576501b368c2d11f2a86d352eecff18054595f54951a；115 payload加manifest共116安全唯一普通成员。root重开全部成员核hash，包内保存实际runner/helper、完整raw、源码清单、实际选路径Git树和独审。CLI二进制与JDK以外pin保存，不把它们虚称包内。
