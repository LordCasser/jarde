# 窄型算术（byte/short/char）巡查（2026-10-05 root，负结果）

## 探针

[fixture/NT.java](fixture/NT.java)（`--release 8`）：byte/char 后缀自增、short 前缀（**窄型无 iinc**——javac 发 iadd+回转 cast）、窄型复合移位（`b <<= 1`/`s >>= 2`）、char 复合加、byte 127 溢出回绕（行为敏感）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- 全部呈现为**显式收窄 cast 形**：`arg0 = (byte) (arg0 + 1)`、`(short) (arg0 >> 2)`、`(char) (arg0 + 1)`——javac `iadd+i2b/i2s/i2c` 回转的忠实展开（byte 算术=int 算术+截断的语义精确呈现）；
- wrap 溢出回绕正确（127+1→-128）；
- 行为 `42/306/b/6/-3/y/-128` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。窄型算术族（前后缀/复合移位/复合加/溢出）确认覆盖。
