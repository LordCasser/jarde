# 旧式手动资源管理 + 深 null 守卫链巡查（2026-10-05 root，负结果）

## 探针

[fixture/MR.java](fixture/MR.java)（`--release 8`）：**pre-TWR 手动资源**（`Reader r = null; try{ r = ...; ...; } finally { if(r != null){ r.close(); } }`——java8 存量代码最常见资源形，与 TWR 字节码形状完全不同）、**深 null 守卫链**（`n != null && n.next != null && n.next.next != null` 三层短路嵌套字段）。

## 结果：**健康，无缺口**（quotes=0）

- **legacyClose 完整恢复**：`local1 = null` 前置、try 内赋值+读+拼接 return、**finally 含 null 守卫+close 完整呈现**（BCI 级 finally 复制正确归位）——pre-TWR 形不需要 TWR 片的几何扩展（finally 机制覆盖）；
- **deepGuard**：三层短路呈现为嵌套 if（语义等价——&& 链的既有规范化）；
- 行为 `read:65/deep/none/none` 逐行 IDENTICAL（渲染以池形 `MR$Node`→`Node` 改写编译——池形名债务第 5 数据点，非行为差异）。

## 处置

负结果归档，不立 spec。旧式资源形确认由 finally 机制覆盖（与已立项 TWR 片边界清晰：TWR 片只处理 javac9+ 关闭抑制几何）。
