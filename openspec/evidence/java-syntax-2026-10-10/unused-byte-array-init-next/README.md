# Unused local byte-array initializer baseline

This private baseline isolates the active source-presentation assertion in
JADX `TestArrayInit.test`: `TestCls.test()` declares an unused local byte array
initialized from `{ 10, 20, 30 }` (`TestArrayInit.java:14-17`, assertion at
`25-29`). The input keeps only that method shape. It deliberately omits the
separately covered `test2()` instance-field assignment.

`Runner` calls `test()` and prints `done`. That confirms the complete source
compiles and executes; the local array is not observable through this Runner.
The collector records the returned `test()V` method text and local array
initializer facts without asserting that a particular decompiler must retain
the dead local, and without treating runtime output as proof of element values
or allocation retention.

The prepared collector uses the frozen Java 8/23 JDK manifest, JADX 1.5.6,
and the already frozen instance-array Jarde CLI. It prepares two original,
four JADX, and four Jarde full-class compile/runtime legs, with independent
class directories, empty classpath/sourcepath, original target source compiled
unchanged, and `-Xverify:all`. No collector or toolchain has been run for this
fixture yet. The baseline will be reviewed before root executes it.

## root 实际验收

04:43 UTC实际collector v1完成35命令，原/JADX/Jarde共10条完整编译运行腿均退出0。collector沿用了byte返回的错误描述符`()[B`，导致两条原case success=false和3条failure；原manifest不改，不冒称其成功计数为10。04:47 UTC root实际独立verifier v3退出0，重新核`<init>()V`/`test()V`的physical census及全部原指令BCI，接受127闭合文件，状态accepted-baseline-with-recorded-collector-error；所有10条隔离源码编译/运行与原raw一致，Jarde默认/all正文一致。执行/失败历史在results，完整结果unused-byte-array-init-independent-acceptance-v3.json。v2实际set/list比较错误也原样保留。

Jarde四份输出均保留`byte[] local1 = new byte[]{10, 20, 30};`，JADX两种profile保留`byte[] bArr = {10, 20, 30};`。这是合法声明写法差别，未发现省略分配或元素，不建立新产品机制；Runner只观察方法返回完成，内容与来源由物理指令/source map和完整输出核对，不能把done当元素值行为oracle。71/612分母和EM18整单元状态不变。
