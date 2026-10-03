# 语句位构造巡查（2026-10-03）——unconsumed-construction 边界的过宽面

实例初始化块巡查引出（主线 `6be15d9b`；init 块平铺进 ctor 的既有形态**健康**——B5 构造器体完整恢复）。固定转录 [fixture](fixture/)（B5 复合 + B6 最小判别；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 判别矩阵（单变量）

| 场景 | 主线 Jarde |
| --- | --- |
| 消费位构造（赋值 `int r = new B6(3).n;`、实参 `println(new B6(9).n)`） | 恢复 |
| **语句位构造（结果不被消费）**：`new B6();`、`new B6(7);`、链式 `new B6(new B6(1).n);`、复合 B5.main（三 new 语句） | 整方法拒绝（"an allocation … is presented only where a rule proved what it builds"）→ main 空体，剥离引注后**可编译但丢全部 ctor 副作用**（行为静默差面，同 do-while 风险族） |

## 根因

`refuse-unconsumed-construction-invokes` 切片把语句位 new 收紧为"仅实参依赖真实调用的构造"——其目标是 CST（实参效果先于构造）序保真。但**无实参/纯常量实参/实参为已证消费链**的语句位构造没有任何序问题：呈现 `new B6();` 即逐字忠实。收紧边界在这些无害形上过宽，副作用仅存在于用户 ctor 体内（B5 形——行为可见）。

## 处置方向

`recover-statement-position-news`（窄切片）：语句位构造呈现为表达式语句——判据为既有 new@1 构造证明（ctor 解析、实参按既有通道）+ 无额外序敏感形（实参均为常量/局部直读/已证嵌套构造）；呈现 `new X(args);`。原切片保护的"实参含真实调用"CST 形保持拒绝（其反例不动、拒绝码不变）；B5.main 与 B6 的 argless/withArg 恢复且行为一致。**2026-10-04 root 裁决补充**：`chained`（`new B6(new B6(1).n)`，实参含 `getfield`）**保持拒绝**——getfield 触发字段声明类 `<clinit>`，其安全需"声明类已初始化"证明（独立事实），不为罕见形引入类初始化推理；登记为遗留边界。

原 class 为行为基准。
