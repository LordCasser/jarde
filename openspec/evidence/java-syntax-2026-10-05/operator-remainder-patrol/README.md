# 运算符残留巡查（2026-10-05 root）

## 健康面（负结果）

[fixture/OP2.java](fixture/OP2.java)（`--release 8`）：
- **移位复合三形**（`<<=`/`>>=`/`>>>=`，int+long）全恢复（展开形——字节码本就无复合 opcode）；
- **NaN/±Infinity 常量**：字节码是 `ldc // float NaNf` **常量池直载**；十六进制浮点**无法表达非有限值**，jarde 以求值等价的除法式编码（`0x0.0p-126f / 0x0.0p-126f` = 0/0 = NaN；`1.0/0.0` = +Inf；`-1.0/0.0` = -Inf）——**行为精确验证**（NVT 探针：`NaN/Infinity/-Infinity`）；
- **负零** `-0.0f`：`-0x0.0p-126f` 直书且**符号位保真**（`1.0f/x < 0` 验证 true）。

## 发现：条件内赋值（第 13 个新证缺口，copy 值家族第 4 员）

`(x = x + 1) > 0` 拒——javac 发射 **dup-store 舞蹈**：`iadd; dup; istore_0; ifle`（dup 后 istore 吃一份、ifle 吃另一份——"copy at BCI 3 has no proved local assignment"）。`condAssignOld`（复合+短路链）与 `main`（级联）同因。

**jadx 有解**：`return i + 1 > 0;`——参数槽赋值不可观察，jadx 丢弃 store（**不可观察赋值消除**）。copy 值家族至此四员：数组 dance / 后缀旧值 / putfield 链 / **dup-store**。

## 处置

第 13 个新证窄缺口（呈现域：dup-store 的双读者呈现——istore 一份+条件一份；可参考 jadx 不可观察赋值消除，但需保守判据）。登记 + 立窄片。
