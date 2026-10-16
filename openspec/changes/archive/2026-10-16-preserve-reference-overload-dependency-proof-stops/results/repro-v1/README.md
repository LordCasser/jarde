# EM11 实际观察基线

capture.tar.gz：1024592B，SHA256 e3304b9e8cf59c26ed6418af254d8b79d18fbf340000053a8293c193809a54e8；49 payload 加 manifest，共50个安全唯一普通成员。3份实际执行收据绑定28份raw，44条外置历史input pins保持原样。缓存打包后逐字节回读，独审293项检查/0错误；manifest为内嵌清单的原样副本。脚本仅供历史审计，不能覆盖旧目录重跑。

基线为产品aadf71与冻结CLI c2db4721…，归档main f30df5d9保持相同4905产品输入。screen实际8命令：未改七Java由javac23 --release8编译，7class SHA与历史一致，新5001B STORE JAR SHA7f424cfedc412c210679cf2d36377bc94b1fb599217b41e8dd4e6fbf0129b9e2；原Java23 Runner固定五行219B。InputHierarchyCalls run 的物理NEW BCI0、constructor BCI4、invokestatic take(HMid) BCI7；不要混淆source-map producer锚点4。CLI4096/1/2/3/4实际exit0/4/4/0/0，cap1/2本已Partial，不是false Complete反例。

真实GDB两次版本和两次batch共4命令，8份GDB raw及4份inferior raw；cap1先reference-overload/raw-reader拒付、后typed snapshot拒付，cap2只有typed snapshot拒付。GDB wrapper0与inferior4分别核实；观察脚本只断点/打印栈/continue，没有改返回、预算或目标状态。只归一10条实际usage.elapsed_millis叶子后wholeJSON与普通同cap相等，所有limits完全保留，stderr逐字相同。

动态仅证明拒付栈位置和顺序；未打印Error/Report返回或catch执行。直接错误吞没、Report→None是exact-source静态风险，取消/期限/平台缺失停止未动态覆盖；此前未fresh重编完整Jarde生成七类。无本片实现、未来CI或完整语义等价结论，前片aadf71 CI不能完成本片。原Runner/oracle/生成输出未修改，外置CLI/JDK原生二进制不收入包。
