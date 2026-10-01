# 嵌套枚举常量体折叠：三方行为对照记录（任务 3.2）

2026-10-01。三方 = 原冻结 class / 固定 JADX（`/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`，1.5.6-dev）/ Jarde 修复后 `class-source` 重编。每方 `javac --release 8` 编译后以 `java -Xverify:all` 运行同一反射 Runner（避开 javac 源级 `p.Holder2$OpAbs` import 限制），比较 `values()` 顺序、`name()`/`ordinal()`、`apply(3,4)` 结果与 `valueOf` 恒等。

## `nested-iface-abs.jar`（SHA 见 sha256.txt，未改动）

| 格 | 原 class | JADX | Jarde 重编 |
| --- | --- | --- | --- |
| `p.Holder2$OpAbs` | `ADD:0:7` / `MUL:1:12` / `valueOf: true` | 同 | 同 |
| `p.OpIface` | `ADD:0:7` / `MUL:1:12` / `valueOf: true` | 同 | 同 |

JADX 输出把 `Holder2$OpAbs` 折叠进 `p/Holder2.java`（`public enum OpAbs { ADD { … } … }`），编译运行行为一致；Jarde 输出为独立 `public enum Holder2$OpAbs { ADD { public int apply(int arg1, int arg2) { return arg1 + arg2; } } … }`（`p.Holder2_OpAbs-fix.jarde.java`）。

## `N2-ops-fix.jar`（自行打包，含全部兄弟类；SHA 见 sha256.txt）

源 = 巡查冻结转录 [../../fixture/N2.java](../../fixture/N2.java)（`javac --release 8 -g:none` 重编打包，含 `N2`、`N2$1`、`N2$IOperation`、`N2$Operation`、`N2$Operation$1/$2`）。

| 格 | 原 class | JADX | Jarde 重编 |
| --- | --- | --- | --- |
| `N2$Operation` | `PLUS:0:7` / `MINUS:1:-1` / `valueOf: true` | 同 | 同 |

JADX 将默认包类包装为 `defpackage.N2`（Runner 按 `defpackage.N2$Operation` 反射，行为同）；Jarde 输出 `N2_Operation-fix.jarde.java` + `N2$IOperation` 恢复文本，独立编译运行。

## 连带覆盖登记格 `variants-fix.jar` 的 `p.TwoLevel$Middle$Deep`

| 原 class | JADX | Jarde 重编 |
| --- | --- | --- |
| `X:0:1` / `Y:1:2` | 同 | 同 |

## 汇总

四方（两嵌套验收格 + 顶层两格同 jar 复验 + 连带登记格）`apply` 路径逐值一致，全部 `java -Xverify:all` 通过。Runner 文本、逐方输出与产物 SHA 由执行时记录固化于本目录 [sha256.txt](sha256.txt)。
