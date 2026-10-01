# 用户类层级实参上转型巡查（2026-10-02）——两闭集切片登记升级路径的首个真实触发

接口/实现域巡查（主线 `c6bebddf`）。固定 [fixture](fixture/)：`I1.java` 源、含全家族的 `fam.jar`（SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）、行为基线 orig.out（`hi:d`/`hello:d`/`static`/`hello:v`）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| `Greet` 接口本身（default 方法体、static 工厂、匿名实现类 `Greet$1`） | 全部完整恢复 |
| `En implements Greet` 覆写调用（`new En().hello("d")`） | 恢复 |
| **`viaInterface(new En(), "v")`：实参 `I1$En` → 形参 `I1$Greet`** | 拒绝："declared `I1$Greet` presents `I1$En` but … no safe reference conversion evidence"（BCI 39）→ 行为差（`hello:v` 缺失） |
| jar 家族输入 | 同失败——快照含 `I1$En` 字节但无证明通道消费 |

## 根因与定性

`build.rs` 转换分派的既有回答（Object 目标/同名/数组/overload 证明/平台闭集×2）都不覆盖**用户类→用户接口/父类**。这是 `recover-throwable-wrap-arguments`（用户异常类→Throwable）与 `recover-platform-collection-widening`（MyList→List）两处登记的 **"resolution 层证明升级路径" 的首个真实触发案例**：任何用户实现类传给以其接口/父类声明的形参（策略/回调/集合注入……反编译最高频形态之一）。

关键架构事实：**`I1$En.class` 的 header 自带 `implements I1$Greet`**——同快照内两类均为物理类，层级事实可从被分析工件自身的字节读取（extends 链 + interfaces 数组），不需要 classpath 猜测。JVM verifier 保证可赋值，源级安全由同快照 header 链证明。

## 处置方向

`recover-snapshot-hierarchy-widening`（大颗粒 MVP）：同快照双方均物理定义的用户类层级实参上转型——按 BCI 收集 widening 证明（呈现类型 T、要求类型 U 均在本快照有物理类时，沿 T 的 extends/implements 链有界 walk 达 U 即证），分派处消费；`cast_argument` 保留要求类型拼写。单边在快照外（含平台目标如用户异常→Throwable）不在 MVP，保持拒绝并继续登记（下一触发时扩单边）。I1 家族四路径逐字一致；两平台闭集与全部既有负例零回退。

原 class 为行为基准。
