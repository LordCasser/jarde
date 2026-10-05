# 显式类型见证/数组类字面量/枚举 values() 巡查（2026-10-05 root，负结果）

## 探针

[fixture/TW.java](fixture/TW.java)（`--release 8`）：`Collections.<String>emptyList()`（显式类型见证）、`int[].class`（原始型数组类字面量）、`String[][].class`（二维引用数组）、枚举内 `for(Color c : Color.values())` SB 累积。

## 结果：**健康，无缺口**（quotes=0）

- **类型见证正确擦除**：`Collections.<String>emptyList()` → `Collections.emptyList()`（见证是纯编译期信息，字节码不可见=忠实呈现）；
- **数组类字面量逐字**：`int[].class` / `java.lang.String[][].class` 精确（ldc of class 常量）；
- **枚举内 values() 遍历**：`local1 = values(); for(TW$Color local4 : local1)` 恢复（合成 $values 桥按既有机制）；
- main 四段拼接恢复；行为 `0/[I/[[Ljava.lang.String;/RED,GREEN,BLUE,` 逐行 IDENTICAL（池形伴生以字面 `$` 名类文件拼接验证——**池形名债务第 4 数据点**：跨文件引用 `TW$Color.all()` 需字面 `$` 类名文件）。

## 处置

负结果归档，不立 spec。
