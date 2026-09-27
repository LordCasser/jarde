# CF-18 区域归属分析的主线复核

root 用合并态 CLI SHA-256 `0c299a1c0143259d5cfe2580ad2ba30e101168ba5c1cec972874952a366086f4` 从缩小输入重建完整 Jarde 源码，与归档 [源码](HandlerLoopProbe.jarde.java)逐字节相同。原 class SHA-256 `2cce0d5077dd04cd3c90c790fbcf3e3a5f54a96250b6185b92dfa8118367ec0e`；原/JADX 完整源码以 `javac --release 8 -g:none` 重编、`java -Xverify:all` 均输出 `4:110`，Jarde 完整源码因 `run()I` 缺返回不能编译。

root 核对生产门槛：`proved_loop_catch_joins` 确实对外层异常表完整区间的每个 block 要求普通流的循环头支配；内层 handler 仅经异常边进入，故这一条件不成立。`guard::nests` 另要求嵌套候选各 handler 只属于本行，而外层 handler 被拆开的多行共同使用。两道门分别承担循环回接与词法异常区所有权，不能只去掉 irreducible 诊断；固定 JADX 在完整 CF-18 样本中错放外层 catch 的运行反例说明了宽松重排的风险。下一步须先证明分段同 handler 行的唯一词法 owner 和跨 handler 的块归属，再建立可实施的 OpenSpec；本轮不把未获证设计派发为代码修改。
