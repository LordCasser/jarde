# 物理 `Exceptions` 多项顺序

## 结论

Java 8 classfile 的 `Exceptions` 项顺序会出现在反射 API 返回的异常数组中。三组由 `javac --release 8` 生成的空正文方法确认：原始 `javap -v` 和 `Method.getExceptionTypes()` 都与源码声明同序。`-g` 与 `-g:none` 两种编译均得到相同结论。JADX 1.5.6 对三组都输出 `SQLException, IOException, ReflectiveOperationException`；A 和 B 因此改变了顺序。JADX 源码完整重编后，反射也返回其改写后的顺序。C 的输入刚好与 JADX 输出相同。

工作树 Jarde 的 class-source 在 `-g` 与 `-g:none` 下均保留 A、B、C 的物理顺序；六份输出完整重编，重编后反射异常数组也全部与原始顺序一致。复放期间曾遇到共享工作树并行编辑中的短暂编译失败，收到共享编译检查点通过后，本脚本重新完整执行并通过。审计未修改生产代码。

## 可复放样本

[`fixture/throwsorder/A.java`](fixture/throwsorder/A.java)、[`B.java`](fixture/throwsorder/B.java)、[`C.java`](fixture/throwsorder/C.java) 分别声明以下顺序：

| 类 | 原始声明顺序 |
| --- | --- |
| A | `IOException, SQLException, ReflectiveOperationException` |
| B | `ReflectiveOperationException, SQLException, IOException` |
| C | `SQLException, IOException, ReflectiveOperationException` |

[`Reflect.java`](fixture/throwsorder/Reflect.java) 打印 `Method.getExceptionTypes()` 的顺序。[`replay.py`](replay.py) 在系统临时目录编译输入，通过 `javap -v` 读取物理 `Exceptions` 顺序，分别反编译并完整重编 JADX、Jarde 类，再以 `java -Xverify:all` 运行反射检查。临时 classfile 与独立 `CARGO_TARGET_DIR` 由 `TemporaryDirectory` 自动清理。

实测环境为 OpenJDK 23.0.1（`--release 8` classfile，major version 52）与 JADX 1.5.6。通过命令为 `python3 openspec/evidence/java-syntax-2026-09-25/throws-order/replay.py`；脚本为每轮建立私有 `CARGO_TARGET_DIR`，所有临时 classfile、输出目录和 Cargo 构建残留都由 `TemporaryDirectory` 自动清理。
