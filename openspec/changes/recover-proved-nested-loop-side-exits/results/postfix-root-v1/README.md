# Root双JDK三方完整类对照接受

同新CLI 9fc77590ab671ac141dc7fb243e77b7fc103ffc87465c629ca0cf8f43e3da0b3，绑定candidate-cli-build-root-v8完整4707 closure与clean。固定/受控各29实际命令58 raw，原类2腿、JADX default-none4腿、Jarde default-all4腿均完整源码原样重编、Xverify:all run exit0，stdout/stderr同同JDK原类。JADX1.5.6是现有runtime，源码审阅基准是固定local checkout，二者不冒称同一构建。

固定只能动态覆盖return12；受控10cases独立模型核早return0、runCounts13/1/4、normal12020、zero12000。默认/all完整source与方法map恒同；全部物理BCI、Code envelope/index/flags、准确owner/name/descriptor、UTF8 spans由独立class parser与javap闭合。CLI class_bytes digest只核形状/length与owner绑定，未独立重算BLAKE3。

两checker真实exit0。fixed checker经exec_command执行并显示stdout（receipt已落盘）；controls checker经launcher subprocess保留stdout/stderr/raw和命令exit，不能冒称fixed也有子进程raw文件。原基线compile1和中间脚本/栈失败未覆盖。仅本窄片接受，不关闭CF11/71整单元；fresh workspace/Clippy/own CI尚待。
