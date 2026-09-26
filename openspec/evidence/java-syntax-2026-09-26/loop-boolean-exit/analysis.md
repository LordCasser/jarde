# 循环测试链：已恢复文本改变退出语义

`LoopBool.java` 以 `javac --release 8 -g:none` 编译，主 class SHA-256 `cc727795087999f893efffd1f98ccafb93e25f988b62689850af51be1f25914f`。本目录保存同一 class 的 `javap.txt`、JADX 1.5.6 输出 `jadx.java` 和主干 `af138e57` 代码构建 CLI 的 `jarde.java`。`andWhile` 的字节码在 BCI 3 与 7 两次 `ifle 23`，任何一次失败都会退出循环；体更新是 BCI 10–20。

源码和 JADX 都写 `while (a > 0 && b > 0)`。Jarde 当前却写 `while (arg0 > 0) { if (arg1 > 0) { ... 更新 arg0、arg1 ... } }`，没有 else/break；当 `arg0=1,arg1=0` 时，原 class 在 `-Xverify:all` 下立即返回 `0`，把 Jarde `andWhile` 方法原样放进 `LoopAndRecovered.java` 后同样编译成功，但执行 2 秒仍未终止。这不是外观差异，而是 `Structured`、无 fallback 的错误程序。`orWhile` 与混合条件当前诚实地报 `jre_region_loop_shape`，没有输出错误循环；JADX 对 `orWhile` 写 `while(true)` 加退出判断，也没有复合 `while` 头。

架构根因在 `region.rs::header_tested_loop`：头部 BCI 2 的一条外边和一条内边足以先建 `Loop`；内边 BCI 6 的条件分支又把指向循环出口 BCI 23 的臂当成普通单臂 `if` 的空臂，`Frame::arm` 停在该 join，Builder 只写 `if (arg1 > 0)`，未写 `LoopBreak`。`covers` 仅检查自然循环集合中的块都被访问，检查不到退出边的语义已丢。至少需要先让这种未表示的内部分支出口拒绝或写成已有的循环 `break`，才能安全构造更好的 `&&`/`||` 复合头。不能仅让 emitter 合并两个条件文本。
