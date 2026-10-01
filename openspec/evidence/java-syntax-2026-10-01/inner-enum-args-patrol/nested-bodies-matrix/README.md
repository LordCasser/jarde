# 常量专属匿名体：顶层已闭合与嵌套名剩余差距（2026-10-01 判别矩阵）

本文件为 [inner-enum-args-patrol](../README.md) 的 N2 形态补充取证。**更正**：N2 在巡查中的"折叠失败"结论需要限定——已合入的 [recover-proved-enum-constant-bodies](../../../../changes/recover-proved-enum-constant-bodies/)（2026-09-26 root 验收，`Op`/`Mixed`/`Plain` fixture）**已闭合顶层名枚举的常量体折叠**（`demo.Op` 于 `toplevel-op.jar` 完整折叠 `ADD { … }, MULTIPLY { … }`，子类并入常量体）。剩余差距为**嵌套名**形态，判别矩阵如下（[产物](.)，JAR 与恢复输出 SHA 见 `sha256.txt`，主线 `16578b78`）：

| 场景 | 枚举自身名 | 折叠 |
| --- | --- | --- |
| `demo.Op`（抽象方法常量体，旧切片 fixture 重放） | 顶层 `Op` | ✅ `ADD { public int apply… }` |
| `p.OpIface`（implements 接口常量体） | 顶层 `OpIface` | ✅ 折叠 |
| `p.Holder2$OpAbs`（抽象方法常量体） | 嵌套 `Holder2$OpAbs` | ❌ 逐字段 |
| `N2$Operation`（implements 接口，巡查 fixture） | 嵌套（默认包） | ❌ 逐字段 |

**判别变量 = 枚举自身二进制名含 `$`（嵌套枚举）**——与抽象/接口实现无关。`TestEnumsInterface` 的 `TestCls.Operation` 正是嵌套枚举，故其账本挂起成立；`TestInnerEnums.Numbers` 同为嵌套（其常量体形态待本片后同批覆盖）。方向：常量体折叠的子类名派生/匹配在父名含 `$` 时错位（子类 `Holder2$OpAbs$1` 的父名回切或 InnerClasses 关系解析取错段）。

处理：`recover-nested-enum-constant-bodies` 窄切片（判别矩阵即正反 fixture）。原 class 为行为基准。

## 更新（2026-10-01，`recover-nested-enum-constant-bodies` 实施后）

上表两嵌套格已随该 change 翻转为折叠；两顶格局逐字不变。首个拒绝点为常量体折叠整组门中"synthetic 访问桥 marker 参数类型 == 常量匿名子类名"的顶层形态名绑定（嵌套枚举 javac 取外围类合成匿名类作 marker，如 `p/Holder2$1`）；修复改为桥的结构事实绑定（桥体委托 + 子类按桥描述符空 marker 调用），义务证明零放宽。定位记录、负例/变体前后（人为 `$` 顶层名不误伤、`A$B$C` 连带翻转登记、嵌套+抽象/接口双形保持逐字段）、三方 `java -Xverify:all` 对照与产物 SHA 见 [refusal-point-fix.md](refusal-point-fix.md)、[variants-before-after-fix.md](variants-before-after-fix.md)、[threeway-fix.md](threeway-fix.md)、[sha256-fix.txt](sha256-fix.txt)；冻结 jar 两行（上文 SHA）未改动。
