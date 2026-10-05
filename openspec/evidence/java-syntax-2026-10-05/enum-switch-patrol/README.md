# 枚举 switch 巡查（2026-10-05 root，负结果——与 jadx 平级）

## 探针

[fixture/ES.java](fixture/ES.java)（`--release 8`）：穷尽枚举 switch 无 default（尾 return 兜底）、共享尾变量 + default、**嵌套**（枚举 switch 内嵌 int switch）。javac 为枚举 switch 生成合成 `ES$1.$SwitchMap$Color` 间接层（经典难点）。

## 结果：**健康，与 jadx 同构**（quotes=0 / notrec=0）

- 合成 `$SwitchMap` 间接层被解析为 **`switch (arg0.ordinal())` 数值 case**——与 jadx 呈现**完全同构**（[results/jadx-ES.java](results/jadx-ES.java) 逐形对照）；
- 无 default 源（穷尽 + 尾 return）正确合成 `default: return "?";`（语义等价：尾 return 是 switch 不可达时的兜底，合成 default 携带它最忠实）；
- 共享尾变量（`local1 = …; break;`）与**嵌套 switch**（枚举 case 内 int switch）完整恢复；
- 行为：渲染源集（参数类型补全 `ES.Color` 限定后）`javac` exit 0、`java -Xverify:all` 输出 `r/0/r1/g` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。枚举 switch 三形（含嵌套）确认覆盖且与 jadx 平级。
