# S1 三方对照（recover-caught-value-argument-typing，变更后）

日期 2026-10-01；JDK 23.0.1（`javac --release 8` 编译所有 Java 输入）；`java -Xverify:all`。
固定 JADX：`/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`（默认参数）。
Jarde：本 worktree 修复后 `target/debug/jarde-cli class-source --policy single-class`。

注入异常路径 = `touch` 抛 `IllegalStateException("boom")` 的同类（[`../threeway/S1X.source.java`](../threeway/S1X.source.java)，
twrHelper 字节码形状与 fixture 相同）；Jarde 对 `S1X.class` 的恢复文本 =
[`../threeway/S1X-typing.jarde.java`](../threeway/S1X-typing.jarde.java)。

## 行为对照

| 来源 | 正常路径 | 注入异常路径 |
| --- | --- | --- |
| 原 class（fixture / S1X.class） | `done` / `done` | `tag:boom` / `tag:boom` |
| 固定 JADX → javac 8 重编 | `done` / `done` | `tag:boom` / `tag:boom` |
| Jarde（修复后）→ javac 8 重编 | `done` / `done` | `tag:boom` / `tag:boom` |

六次运行全部退出码 0，`-Xverify:all` 无告警；三方一致。

## SHA-256

```
e372f219aa6b556e6717b10648d99ff3d5f4bdaca5a4b4f68233d2f157b3c765  ../fixture/S1.class（原 class，三方输入之一）
56f4c39f7ad7905c8a72c11d14e48d76e27f69c42bdc5ba415240b0e2f51a807  ../threeway/S1X.class（注入异常路径原 class）
dc4b31fdc9e64d0a7241b9a7688446a800c1f2759337f8eaf225f3b75bfd0044  S1-typing.after.java（Jarde 恢复文本）
7f7e372be2d56397f93dc503edf9b1a45b0f4b9e8e0690987a99b09000515b20  ../threeway/S1X-typing.jarde.java（注入路径 Jarde 恢复文本）
bf65a35f02477d060c34a5cbea0e0a3c23182344cc04936df562d7a70f4e1533  JADX S1.java（正常路径，去 `package defpackage;` 行后重编）
7c2300ad5b3372b8d07f7a09136593d086fce8c3b1de750bb5f34c84f9615119  JADX S1X.java（注入路径，同上处理）
```

命令（可重放）：

```
javac --release 8 S1.source.java                       # fixture/S1X.source.java 同法
java -Xverify:all -cp . S1                              # 各方重编产物分别执行
<JARDE> class-source --input S1.class --policy single-class --class S1
jadx -d <out> S1.class                                  # 固定 JADX，默认参数
```
