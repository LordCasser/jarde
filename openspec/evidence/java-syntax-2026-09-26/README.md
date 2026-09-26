# 2026-09-26 语法巡查：方法引用 / 精确重抛 / 多资源 TWR / char switch

自写 `Patrol.java`（javac --release 8，`-g` 与 `-g:none`），jadx 1.5.6 与 jarde `class-source` 三方对照。本文件只固定缺口定位；验收以各 change 为准。

## 定位的缺口

| 形状 | jadx 1.5.6 | jarde | 依据 |
| --- | --- | --- | --- |
| 精确重抛 `catch (Exception e) { log(e); throw e; }` + 窄 throws | 恢复为 try/catch，但 `throws` 宽化为 `Exception`（jadx 偏离） | 整方法 `jre_guard_finally_copy` 引用 | `recover-precise-rethrow` |
| 多资源 TWR（两资源） | 展开为手写嵌套 try + close + addSuppressed（jadx 不还原 TWR） | 整方法 `Unproven::RangeEnd` 引用 | `recover-multi-resource-twr` |
| SAM 装箱适配（`Supplier<Integer>` ← `this::supply`，impl `()I`） | 完整呈现箭头 | 整方法引用（「outside the proven identity, Object check, or Object upcast cases」） | `recover-boxed-sam-adaptations` |
| `int[]::new` 数组构造器引用 | 呈现 | 站点拒绝（impl `(I)[I` vs instantiated `(Integer)[I`） | `recover-boxed-sam-adaptations` |
| char 局部为 switch 选择子（无 LVT 类型决策为 int） | `case 'A':` | `case 65:`（语义等价，拼写） | 归 2c.12 / 局部类型决策家族，不开新 change |

## 字节码事实（javap 摘录见 javap.txt）

- preciseRethrow：一行具名 `[0,39) → 42, Class Exception`，handler `astore_2; aload_2; invokestatic log; aload_2; athrow`。javac 的 finally 合成拷贝只由 `catch_type==0` 行进入——`guard.rs::finally_copy` 不读该字段是误判根因。
- multiResource：五行 `Throwable`（内层行 `[11,23)→44`、外层行 `[5,33)→71` 等）；`twr` 层级链可建两层，但几何要求外层行止于内层 handler span 末，与 javac 嵌套语句几何不符。
- 方法引用站点：invokedynamic 的 BSM 参数携带 instantiated MethodType（`()Ljava/lang/Integer;`）与 impl MethodHandle（`supply()I`），适配由 metafactory 链接期 asType 完成，无合成适配器。

## 第二批（Patrol2）：register 已覆盖，无新 change；冻结输出发现一处 2.2 违规

| 形状 | jadx | jarde | 归属 |
| --- | --- | --- | --- |
| assert 带消息（`$assertionsDisabled` 守卫呈现、`new AssertionError` 路径引用） | 呈现 | 部分呈现+引用 | `project-proved-assert-statements`（0/9） |
| do-while + continue（闩锁跳转） | 呈现 | 整方法引用（`canonical block at BCI 2 … more than one owner`） | `recover-do-while-body-transfers` / `refuse-overlapping-region-ownership` |
| 嵌套三元赋值后读取 | 呈现 | 拒绝+下游 `t` 未声明引用 | `recover-conditional-values`（2.4） |
| **多捕获 catch 体（条件值返回）** | 呈现 | 冻结的修前输出中 catch 头呈现、体为空、被丢语句无引用 | umbrella 2.2 第三负例已修复：处理器体证明不了时体是 BCI 引用，不得空块 |
| 位运算混排/`>>>`、浮点比较、long 常量、foreach、字符串 switch、static synchronized | 呈现 | 完整呈现（无引用） | 已覆盖，无缺口 |

## 第三批（Patrol3）：无新 change；一处既有 change 回归线索

| 形状 | jarde | 归属 |
| --- | --- | --- |
| 方法内局部类（捕获参数+局部，`new Patrol3$1Scaled(factor, offset)`） | 完整呈现，与 jadx 同水平 | 无缺口（局部类内联属 5.3 匿名类家族的更远目标） |
| varargs 声明/遍历/透传/泛型 | 完整呈现 | 无缺口 |
| `instanceof int[]` / `String[]` | 完整呈现 | 无缺口 |
| 实例初始化块 + 双构造器 | 完整呈现 | 无缺口 |
| 多维数组复合赋值 `g[i][j] += 7` | 整赋值引用（诚实） | umbrella 2c.16/2c.23 在途 |
| foreach 缓存声明 `local2 = xs`（被迭代表达式是纯参数读取） | 未剪除的冗余拷贝 | `prune-proved-foreach-cache-declarations`（6/6 已完成）疑似回归，待排查 |

## 文件

- `Patrol.java` / `Patrol.class`：自写场景（SHA 待各 change 实施时冻结登记）。
- `jarde-Patrol.java.txt`：修前 jarde 完整类文本（各方法拒绝原因原文）。
- `jadx-Patrol.java`：jadx 1.5.6 输出（偏离如上表）。
- `javap.txt`：全类反汇编。
- `multi-resource-twr/README.md`: self-contained two-resource fixture, Java 8/current-JDK exception-row layouts, verifier-valid wrong-end negative control, deterministic runtime cases, and available JADX/Jarde comparisons.
