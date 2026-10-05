# switch fallthrough 巡查（2026-10-05 root，负结果）

## 探针

[fixture/SF.java](fixture/SF.java)（`--release 8`）：**双贯穿链**（case 1 无 break 贯穿到 2；case 3 贯穿到 4）+ 空 case 叠加（`case 1: case 2:`）——switch 族最后未测的显式贯穿流。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **贯穿精确还原**：`case 1: s += 10;`（无 break）→ `case 2: s += 20; break;`——贯穿 case 体不带 break、被贯穿目标带 break，控制流与源完全同构；
- 空 case 叠加（连续 `case 1: case 2:`）与 default 恢复；
- 行为验证：`30/20/70/40/-1/12/12/3/0` 逐行 IDENTICAL——**贯穿语义精确**（fall(1)=10+20=30 而非 10；fall(3)=70 而非 30——贯穿链完整执行）。

## 处置

负结果归档，不立 spec。switch 族至此全覆盖（return/共享尾/合并 case/String/int/枚举/嵌套/贯穿/空叠加）。
