# EM-22 整数按位与条件：实际完整类对照

目标来自活动 JADX `TestRedundantBrackets.method3`：保留实例方法的两个完整判断，先执行 `a + b < 10`，再执行 `(a & b) != 0`。没有把首个判断删掉，也没有扩到同文件的cast、instanceof或数组表达式。

05:01 UTC root实际运行collector v1，35命令、双JDK共10条完整源码重编/验证运行腿全部成功（原2、JADX4、Jarde默认/all4）。共用Runner覆盖首guard、mask非零/零、负数与MAX+1溢出；原程序的实际stdout为 `3\n84\n7\n-6\n2147483647\n`，全部重建结果逐字一致。CLI是预算修正后的整数名称v2，具体30文件pins及CLI/meta hash见对应metadata。

05:05 UTC root审查完整delta后实际运行独立verifier v2，退出0并接受128个闭合文件、35条原始命令、10条隔离编译运行腿、唯一原javac23目标JADX jar、完整physical public方法身份/flags/index和原javap的全部BCI。默认/all完整正文相同，实际输出保留 `(arg1 & arg2) != 0`。结果见 `results/em22-mask-condition-independent-acceptance-luna-v2.json`，执行记录见 `results/independent-execution-root-v1`；v1脚本静态发现的未定义函数与JDK flags格式问题保留为未执行的历史版本。

本形状没有已证产品缺口，不增加AST或pass，也不因此宣称EM-22全部追平。观察到既有else-if发射的闭括号缩进不整齐，源码可以完整重编且来源/行为均正确；格式问题单独记为后续审计项，不混入整数名称恢复。
