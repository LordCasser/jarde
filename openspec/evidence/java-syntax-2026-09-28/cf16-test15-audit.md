# CF-16 Test15：别名负例与 finally 输出边界

## 结论

固定 JADX 的 `TestTryCatchFinally15` 是 DEX/smali 输入上的 finally 抽取**负向回归**，不是可计入 Java 语法恢复的正向验收。测试唯一活动方法先调用 `disableCompilation()`，然后只断言错误的 `parcel = Parcel.obtain();` 不出现、`transact` 调用参数使用 `parcelObtain` 且恰好一次。注释中的完整 `try/catch/finally` 不是活动断言；测试没有 finally 形状断言、行为对照或 Java 编译 profile。因此 CF-16 固定清单的 24 个类名中，Test15 应保留在“负向别名回归”分母，不作为 finally 正向语法目标。

这不说明目标形状已被 Jarde 闭合。固定 smali 所表达的一个 Java 8 等价类文件可以通过 `javac --release 8` 编译，且原类与固定 JADX 的正常路径相同；JADX 将正常 `parcel.recycle()` 放进内层受保护区域，并另在 `catch (Throwable)` 中再写一份。正常清理调用成功时只执行一次；若这次清理抛异常，外层 catch 会再次调用它，改变异常路径行为。Jarde 对该等价 classfile 整个目标方法输出 explanation-only，不产生可执行方法体。故“Test15 不是正向断言”与“该语义切片当前仍未恢复”同时成立。

## 固定测试与字节码

固定测试源位于 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally15.java`。第 9–12 行把它明确标为 “Negative test case” 和寄存器合并错误；第 17–33 行只是被注释的预期 Java；第 36–43 行只有一项活动测试，且第 38 行关闭编译检查。

它读取 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/smali/trycatch/TestTryCatchFinally15.smali`，不是 JVM classfile。目标 `test(ILandroid/os/Parcel;)Landroid/os/Parcel;` 的 DEX 地址与唯一异常表几何如下：

| DEX 指令范围/目标 | 含义 |
| --- | --- |
| `[0x4,0xd) → 0x13`, `RuntimeException` | transact/readException 的具名 catch |
| `[0x4,0xd) → 0x11`, catch-all | 同一 try 正文异常路径进入 finally handler |
| `[0x14,0x18) → 0x11`, catch-all | RuntimeException catch 正文抛错时也进入 finally handler |

正常路径的 `parcel.recycle()` 位于第一个受保护范围之外。RuntimeException catch 先回收新分配的输出 Parcel，再重抛；外层 finally handler 回收输入 Parcel 后重抛原异常。DEX 有两个物理输入清理副本（正常路径与 catch-all handler），另有 catch 正文自己的输出 Parcel 清理。固定测试的寄存器别名断言保护 `transact` 第三个实参，不证明上述异常行被还原为 Java 结构。

验证固定输入的命令与结果：

```sh
git -C /Users/lordcasser/workspace/testzone/jadx rev-parse HEAD
# 2fb1b16386941660fda07e9017285aec40fcb37f

cd /Users/lordcasser/workspace/testzone/jadx
./gradlew :jadx-core:test --tests jadx.tests.integration.trycatch.TestTryCatchFinally15
# BUILD SUCCESSFUL; 1 test task executed

jadx --no-res --single-class trycatch.TestTryCatchFinally15 \
  -d "$RUN/jadx-smali" \
  /Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/smali/trycatch/TestTryCatchFinally15.smali
```

该固定安装版 CLI 输出源码 SHA-256 为 `ace725fcbb81e06a8a52dcca873f4d57522f47ad9481d4724eb09c6ff1e9641f`。方法内先出现 `parcel.recycle(); return parcelObtain;`，整个内层 `try` 后再出现 `catch (Throwable th) { parcel.recycle(); throw th; }`。这正是活动断言没有覆盖的边界：若第一处 recycle 自身抛错，第二处会执行。

## Java 8 classfile 对照

smali 测试输入不能直接交给 Jarde（其恢复入口接收 JVM classfile）。为验证这个语义而非伪称 DEX 等于 classfile，我按注释中的方法结构制作了仅用于审计的 Java 8 等价样本：保留具名 RuntimeException catch、其内部输出 Parcel 清理、外层输入 Parcel finally；`Parcel`/`IBinder`/`RemoteException` 为可记录事件的最小 stub。用本机 `javac --release 8 -g:none -Xlint:-options` 编译后，目标 JVM 异常表仍是典型三行几何（BCI 不同于 DEX）：

```text
from 4 to 25 target 32   Class java/lang/RuntimeException
from 4 to 25 target 41   any
from 32 to 43 target 41  any
```

该 classfile SHA-256 为 `f7114f29d3e5e5c927816705e9587787908262f16d8806effcdfbd7e1d6c82f5`。`java -Xverify:all` 原类结果：正常 `obtain;transact;read;in.recycle;return:out;`；transact 运行时异常为 `obtain;transact;out.recycle;in.recycle;throw:IllegalStateException:transact;`；checked RemoteException 为 `obtain;transact;in.recycle;throw:RemoteException:transact;`；readException 运行时异常为 `obtain;transact;read;out.recycle;in.recycle;throw:IllegalStateException:read;`。

对该 Java 8 classfile 运行固定 JADX CLI，得到的关键方法体与 smali 输出相同。为隔离其清理覆盖错误，我将这段发射方法体原样放进一个仅改名的 package-private adapter，并重用原样本 Parcel/Binder stub；adapter 用 Java 8 重编译且通过 `java -Xverify:all`。当输入 Parcel 的首次 `recycle()` 抛一次 `IllegalStateException` 时，原类日志为 `obtain;transact;read;in.recycle;throw:IllegalStateException:recycle;`，JADX 适配输出为 `obtain;transact;read;in.recycle;out.recycle;in.recycle;throw:IllegalStateException:recycle;`。这证明异常发生在 finally 清理调用本身时，JADX 将该调用放入保护范围会触发第二次清理；成功返回的普通路径仍只有一次清理。

用当前主线 `d81df5235891feaf50a3ca9822d41ca2f336299c` 的 class-source 路径读取同一个 Java 8 classfile（单类输入，报告 class digest `4a8572a4715218ee84bae385b8c5060524cfb3c622872fce1aa128dd026e3da9`）。`test(ILTestTryCatchFinally15$Parcel;)LTestTryCatchFinally15$Parcel;` 没有发射语句，标记为 “explanation only”；正文指出 `local 3 crosses a quoted fallback region`。这是安全拒绝，不是一个能运行的 Jarde 重建；对该输出没有声称通过编译或语义对照。

用于复放的核心命令为：

```sh
javac --release 8 -g:none -Xlint:-options -d "$RUN/original" \
  "$RUN/TestTryCatchFinally15.java" "$RUN/Runner.java"
java -Xverify:all -cp "$RUN/original" Runner cleanup

"$JADX_BIN" --no-res --single-class TestTryCatchFinally15 \
  -d "$RUN/jadx" "$RUN/original/TestTryCatchFinally15.class"
# 以 decompiled test 方法体构造只改类名/类型限定符的 adapter，再运行：
java -Xverify:all -cp "$RUN/jadx-adapter:$RUN/original" JadxRunner cleanup

CARGO_TARGET_DIR=/private/tmp/jarde-cf16-test15-cargo-target \
  CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked -q -p jarde-cli -- \
  class-source --policy single-class --input "$RUN/original/TestTryCatchFinally15.class" \
  --class TestTryCatchFinally15 --format text
```

这次 classfile 的完整 `javap -p -c -v` 输出 SHA-256 为 `32ff61e5fa147d771ebd72ad3fcfedfe8e86ac3ba7451d2f97ac9d30c10142d4`；固定 JADX 的 JVM 等价样本源码 SHA-256 为 `67d02546bef5b3fec800a7814b2d434bbdbed9fcbdac44c17a3fb34470528fd5`。所有样本、runner、编译产物和 CLI 报告都在 `/private/tmp/jarde-cf16-test15.*`；Jarde 专用 Cargo target 已用 `cargo clean` 清理。

## 当前 Guard / Region / Builder 接缝

- [`guard.rs::prove_finally_copy`](/Users/lordcasser/.codex/worktrees/plain-enum-arities/jarde/crates/jarde-java/src/guard.rs:2012) 的直线证书接受单条 catch-all 行；Test15 的 classfile 有三行，且有具名 RuntimeException handler 和覆盖 catch 正文的第二 catch-all，故此证书不是入口。
- [`guard.rs::prove_conditional_finally`](/Users/lordcasser/.codex/worktrees/plain-enum-arities/jarde/crates/jarde-java/src/guard.rs:2362) 更窄：必须恰好一条 catch-all 行、void 正常 return、两副本各为固定的一次判空/可选调用序列；与 Test15 的返回 Parcel + typed catch + catch-body cleanup 不同。
- 最接近的证明接缝是三行 `SharedFinally`：[`shared_finally_candidate`](/Users/lordcasser/.codex/worktrees/plain-enum-arities/jarde/crates/jarde-java/src/guard.rs:4093) 对三行会依次试 `prove_shared_join_finally` 与 [`prove_shared_finally`](/Users/lordcasser/.codex/worktrees/plain-enum-arities/jarde/crates/jarde-java/src/guard.rs:3405)。两者要求三份配对清理；旧证书还只接受无参 static void 调用或受限 static int 增量（[`shared_cleanup_span` / `shared_cleanup_copies`](/Users/lordcasser/.codex/worktrees/plain-enum-arities/jarde/crates/jarde-java/src/guard.rs:2562)），并要求两份正常副本有已保存返回或共同 transfer。Test15 是一份正常输入清理 + 一份 any handler 输入清理；catch 自身仅回收输出 Parcel 后抛出。因此既不能把 catch-body cleanup 当成 finally，也不是已有三副本完成形式。
- [`region.rs::shared_finally_body`](/Users/lordcasser/.codex/worktrees/plain-enum-arities/jarde/crates/jarde-java/src/region.rs:4328) 能为受证三行形态构造 typed try/catch 的有界正文；Builder 的 [`Shape::Finally` 分支](/Users/lordcasser/.codex/worktrees/plain-enum-arities/jarde/crates/jarde-java/src/build.rs:12982) 能构造普通 catch-all finally，但只消费已证的 `Finally` 计划与完整结构化正文。Region/Builder 的 AST 承载物可复用，不能绕过前置 Guard 的异常行及完成证明。

## 建议的最窄边界

如果后续要支持注释所述的完整行为，应另立窄证书/完成形态，限定为：同一 protected try 的一条具名 `RuntimeException` 行和一条 catch-all 行；catch-all 还覆盖该具名 catch 正文；正常路径和 catch-all handler 恰有两份输入 Parcel 清理；catch 正文中的输出 Parcel 清理后原异常重抛；两份 finally 清理外部无竞争行、自保护或外部入口；清理抛错时只执行一次并取代先前异常。正常路径返回的 Parcel 值必须与清理副本分开证明、由 load/return 消费。可复用 FINALLY pass、Try/If/Throw/Return AST、source map 与原子提交；不能只放宽三行 `SharedFinally` 的 `shared_cleanup_span`，也不能把 Test15 的活动 alias 断言升格为 finally 正向验收。

该方向是基于目标方法结构和当前证明入口的架构建议，不代表已经实现。最小新验收应同时保留 Test15 原始别名负例，并增加独立的 Java 8 classfile 正例及清理自抛、handler 覆盖范围变化、catch cleanup 改写和异常竞争行等拒绝近邻。
