# 移位操作数的求值位置与异常顺序

Java 8 源码、runner 和固定 class 在本目录。`javac --release 8 -Xlint:-options` 产出的 `ShiftEffectOrder.class` 是 865 B，SHA-256 为 `dfc46d9b9c02591a6a37f32d458e819d03b9b9f4ce8761c7159a1d46b5e695b6`。Jarde CLI 是本地移位实现验收版 `/tmp/jarde-shift-ops-replay/jarde-cli`，SHA-256 `9467c73083d721a51abae84455175980bb7a2f2b38ed29c12c7f073d3672f6a9`。JADX 为本机 1.5.6。三份类（原始 class、JADX 完整类、Jarde 完整类）都通过 Java 8 编译和 `java -Xverify:all`；运行同一份 source-only runner。

| `value` | 原 class | JADX 1.5.6 | Jarde |
|---:|---|---|---|
| -7 | `-7:11972:131972:1:12` | `-7:-28:12972:1:12` | `-7:11972:131972:1:12` |
| 0 | `0:12000:132000:1:12` | `0:0:13000:1:12` | `0:12000:132000:1:12` |
| 7 | `7:12028:132028:1:12` | `7:28:13028:1:12` | `7:12028:132028:1:12` |

每行依次是 `value:inline:separated:throwLeft:throwRight`。完整输出见 `outputs/`。原 class 与 Jarde 输出 SHA-256 同为 `6da438f31d6cf7b801bfe0876d4a73e85cf8c4b24ac77b5e495b49e8cda8dbcd`；JADX 输出是 `a92dbf5b2c1a25152eca5dec22d9b019779061d13eaa05639105a0ca03ecdd17`。

`outputs/javap.txt` 给出实际顺序：`inline` 的两次 `step` 在 BCI 6、11，移位在 14，`events` 字段读在 16；`separated` 在 BCI 6 调用左值、12 执行独立语句、19 调用右值、22 移位，字段读在 24。JADX 把两处字段读都前移到了移位表达式之前，分别写成 `(events * 1000) + (step(...) << step(...))` 和 `(events * 1000) + (iStep << step(...))`。这段 Java 能编译，却改变了副作用的可见时刻。Jarde 保留 `int local = ... << ...; return events * 1000 + local;`，与原字节码一致。两种抛错操作数的事件码也一致：左侧抛错时只记录 `1`，右侧抛错时先记录左侧的 `1` 再记录 `2`，返回 `12`。

重放命令（JADX 输出需要去掉其添加的 `package defpackage;`，不改方法体）：

```sh
javac --release 8 -Xlint:-options -d /tmp/shift-effect-original source/ShiftEffectOrder.java source/ShiftEffectOrderRunner.java
java -Xverify:all -cp /tmp/shift-effect-original ShiftEffectOrderRunner
jadx --no-res -d /tmp/shift-effect-jadx original/ShiftEffectOrder.class
sed '/^package defpackage;$/d' /tmp/shift-effect-jadx/sources/defpackage/ShiftEffectOrder.java > /tmp/ShiftEffectOrder.java
javac --release 8 -Xlint:-options -d /tmp/shift-effect-jadx-classes /tmp/ShiftEffectOrder.java source/ShiftEffectOrderRunner.java
java -Xverify:all -cp /tmp/shift-effect-jadx-classes ShiftEffectOrderRunner
/tmp/jarde-shift-ops-replay/jarde-cli class-source --input original/ShiftEffectOrder.class --class ShiftEffectOrder --policy single-class --release 8 --format text --output /tmp/ShiftEffectOrder.java
javac --release 8 -Xlint:-options -d /tmp/shift-effect-jarde-classes /tmp/ShiftEffectOrder.java source/ShiftEffectOrderRunner.java
java -Xverify:all -cp /tmp/shift-effect-jarde-classes ShiftEffectOrderRunner
```

这里的结论只覆盖移位表达式的求值位置和异常顺序。捕获放在 runner 中，不借本样本判断 Jarde 的 try/catch 区域能力。`ShiftEffectOrder` 中 `separated` 保存左值再执行独立语句，是有效的旧局部读场景；Jarde 没有把 `step(1, value)` 越过那条语句重新求值。
