# Test15 Java 8 等价类复放

该目录将固定 Test15 的注释方法结构编译成 JVM Java 8 classfile，以便 Jarde 的 classfile-only `class-source` 可以读取；它没有把 JADX 的 DEX 输入冒充为原 classfile。`original/TestTryCatchFinally15.java` 是目标方法，`original/android/os/` 是最小 Android stub，`probe/Runner.java` 为五条可观察路径提供副作用与抛错点。

`replay.sh` 从固定 JADX checkout 读取原测试 smali，检查 checkout HEAD、smali SHA-256、JADX smali 输出 SHA-256、等价 JVM class SHA-256 和 JVM 异常表。它用固定 JADX 输出构造 adapter，只移除无关 blank-final 字段限制并增加 binder 构造器；脚本核目标 `test()` 的方法文本在 adapter 中逐字不变。原类与 adapter 分别通过 Java 8 编译及 `java -Xverify:all`，结果逐行对照 [`expected/original.txt`](expected/original.txt) 和 [`expected/jadx.txt`](expected/jadx.txt)。最后脚本用新建的 `/private/tmp` Cargo target 构建当前 Jarde CLI，验证目标方法 explanation-only 拒绝，并在退出时执行 `cargo clean` 与清理临时目录。

复放需要本机 `javac`、`javap`、Python 3、`rg`、固定 JADX 安装版及 Cargo：

```sh
openspec/evidence/java-syntax-2026-09-28/cf16-test15-equivalent/replay.sh
```

设置 `KEEP_OUTPUTS=1` 会在 Cargo target 清理后保留其他诊断输出目录。固定默认值为 JADX checkout `/Users/lordcasser/workspace/testzone/jadx`、安装版 `jadx-cli/build/install/jadx/bin/jadx`；可用 `JADX_ROOT`、`JADX_BIN`、`JARDE_REPO` 指定位置，但 pin/hash 检查仍会执行。

固定期望值：

- Test15 原始 smali SHA-256：`438b7e88987722ae4e015fb7eb475a291f8384ed37ca4f39bfcc1de42b3833b6`。
- 原 Java 8 等价 class SHA-256：`880a6934faa48c317538cc37e4dc72c4d1130d8b50b4a78c0656a583565b4ba3`。
- 固定 JADX 对 smali 的源码输出 SHA-256（将 `loaded from` 绝对路径规范化后）：`c26d853656dfd81f54bb8da716aec85805206b414833191aea054659ed712e27`。在固定本机路径下的原始输出 SHA-256 是 `ace725fcbb81e06a8a52dcca873f4d57522f47ad9481d4724eb09c6ff1e9641f`。
- JVM 三行异常表：`[4,25) → 32 RuntimeException`、`[4,25) → 41 any`、`[32,43) → 41 any`。

正常、transact 运行时异常、checked RemoteException 和 readException 运行时异常路径原/JADX 输出一致；只在输入 Parcel 的 finally 清理首次自行抛错时分歧：原 class 只尝试输入清理一次，JADX 先进入内层 RuntimeException catch 回收输出 Parcel，再进入外层 Throwable catch 第二次回收输入 Parcel。Jarde 对原 Java 8 classfile 整个目标方法以 `local 3 crosses a quoted fallback region` explanation-only 拒绝，不生成运行结果。
