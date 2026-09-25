# static initializer 审计证据

这组证据只审计 Java 8 自写样例的 `<clinit>` 收尾，不代表任意初始化控制流都已恢复。普通直线、条件、循环、try/catch、helper 和 `void` 对照由 `root-after/README.md` 记录；本目录补充空初始化、真实提前返回和外部 helper 初始化异常三个边界。

## 命令和输入

输入源码位于 `sources/`，runner 位于 `runners/`。源码和 runner 均使用：

```text
javac --release 8 -g:none -d <temporary-directory> <source> <runner>
java -Xverify:all -cp <temporary-directory> <runner>
```

审计脚本是 `run_boundaries.py`。它把 class、runner 编译产物和 jadx 临时目录放在 `/tmp/jarde-static-initializers-20260923-boundaries/`，把可复核的源码、javap、编译/执行日志复制到本目录的 `boundaries/`，并给每个 subprocess 设置 20 秒上限。恢复使用当前 `target/debug/jarde-cli`：

```text
target/debug/jarde-cli class-source --input <class> --class <name> --policy single-class --release 8 --format text
jadx --no-res -d <directory> <class>
```

JADX 的源码只移除其精确的 `package defpackage;` 行后再编译；该归一化在脚本中可见，原始文本同时保存在 `jadx-raw.java`。所有 class 文件仅保留在 `/tmp`，本目录不提交 class。

## 边界结果

| 场景 | 原 class | 修改后的 class | jarde 整类重编译/执行 | JADX 重编译/执行 |
| --- | --- | --- | --- | --- |
| 空 `<clinit>`（由 `StaticStraight` 的真实 class 精确改为仅 `return`） | `value=7` | `value=0` | 0/0，`value=0` | 0/0，`value=0` |
| 合法 JVM 提前返回（真实分支字节码） | 源码 class `value=7` | `true → value=1`，`false → value=7` | javac 1：两个分支内仍出现非法 `return` | 0/0，两个分支均一致 |
| 外部 `ThrowingHelper.fail()` 初始化异常 | 首次 `ExceptionInInitializerError:IllegalStateException`，再次 `NoClassDefFoundError:ExceptionInInitializerError` | 同上 | 0/0，两次访问结果一致 | 0/0，两次访问结果一致 |

空 `<clinit>` 的 patched class 是 239 bytes，SHA-256 为 `d014fb6bde6f69f22fe9db7d0d290b2e768b89d49749d7340ce395ff38537bf9`；jarde 输出保持静态块为空，并明确写出 explanation-only/no statement。它证明省略唯一尾部 return 不会伪造一条 Java 语句，且完整 class 仍可编译执行。

提前返回的 patched class 是 433 bytes，SHA-256 为 `d551ce9ef25ca82a5bddca32f6b25ac848c9fc57cf8e3921ded62de14a906899`。`javap` 显示 `if` 真分支在 BCI 10 提前返回，假分支从 BCI 11 继续执行后续赋值；两个分支均经 `java -Xverify:all` 验证。jarde 的输出保留两个分支 return，因此 javac 的两个“返回外部方法”诊断是正确的拒绝边界，不能把它归入本次“最外层最后一条 return”投影。

外部 helper 的 source-only 目录只给恢复源码重编译使用，jarde 只输入 `StaticExternalException.class`。恢复主类加 `ThrowingHelper.java` 后与原 class 的同一 JVM 两次访问都得到首次 `java.lang.ExceptionInInitializerError:java.lang.IllegalStateException`、再次 `java.lang.NoClassDefFoundError:java.lang.ExceptionInInitializerError`；这覆盖首次初始化失败和 JVM 对失败类的后续访问，而没有把 helper 的裸 `throw` 当作主类正文重建。

三类边界的逐文件状态、源码、javap 和日志在 `boundaries/StaticEmpty/`、`boundaries/StaticEarlyReturn/`、`boundaries/StaticExternalException/`；汇总是 `boundaries/summary.json`。普通条件/循环/try 的新 CLI 实际整类结果在 `root-after/`，其中条件两次运行分别为 `1` 和 `2`，循环为 `3`，try/catch 为 `7`，helper 为 `11`，普通 void 为 `1`。

## 结论范围

普通 javac 生成的上述可恢复样例，其 `<clinit>` 末尾无值 return 是最外层最后一条终止指令；静态初始化块内的 Java `return` 仍非法。合法提前返回需要单独的控制流结构化，本项只覆盖已经恢复到最外层尾部的终止来源，并把该来源映射到实际闭合 `}`。
