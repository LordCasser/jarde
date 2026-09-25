# 类型限定符被局部名称遮住

2026-09-23，root 用普通 javac Java8 输入构造三份完整类。没有修改任何生成正文，CLI 均为 SHA-256 `7527b03abc1e487121d204672b52043ca5f3aa1148c6d65b136e36f2bbe1f4aa`，每份审计前后验证相同。原 class/JADX 均实际 javac 与 `java -Xverify:all`，结果相等。

## 静态访问出现真实错目标

默认包类名 `arg0` 合法。源方法的参数叫 other，`-g:none` 后恢复为 arg0。字段访问仍输出 `arg0.value`，但此时 Java 将首段认作参数，访问的是 ShadowOther.value。

首份 `arg0.class` 为 359 bytes，SHA-256 `6df3f4ccd7ce4fcbaca22c4a38f38cb0f637ce01d5d7b0df0c4ececf98846363`；6 行中4行错误。原读取3变7，原写入9后双方字段为9:7，恢复结果为3:9。null 与非null 参数均错。另两行本类静态调用保持正确，因为当前调用路径省略本类 owner，写成 pick(1)。

第二份 `external/ShadowExternal.class` 为340 bytes，SHA-256 `ab42b65ce6bdec23cf980037232d946203c2459f07858fb1804cfb6935992445`。这次调用的 owner 为另一个类 arg0，完整恢复保留 `arg0.pick(1)`，转而调用 ShadowOther.pick；原结果11变21。读写也同样错，6行全部不同。两份均零引用且完整javac成功，不能把成功编译或Structured状态当作等价。

## 包前缀同样属于表达式名称

`package-prefix/ShadowMath.class` 保留合法 debug 参数名 java，原源码使用 Math.abs(value)。恢复文本使用 `java.lang.Math.abs(value)`，于是首段被 Object 参数遮住。三项原/JADX相等；jarde零引用而javac失败，未生成jarde执行对照。468 bytes，SHA-256 `d775ee2998d9e6d18cb0a2225ce186c341fb4fa6e04d784af10a53439bc22a0c`。

## 最小架构处理

AST 内 Path 与 Local 已分开，问题出在发出的 Java 文本绑定。NameTable 已有为 blank final 字段服务的 reserved 输入，且共享给普通局部与 free_name。可从本方法使用的同源静态 owner 及字段 claim 收集其类型拼写首段，并入该输入，避免默认名和 debug 名抢占即可；不需要 resolver、import 系统、AST 后处理重命名或新增表达式。

`preserve-type-qualifier-bindings` 已单列规划，生产等待前一任务归还窗口。方法引用的 qualifier 和不能任意改名的真实类字段可能涉及不同边界，本轮没有将它们冒称为已证明可由局部改名全部解决。详见 change 的范围与任务。
