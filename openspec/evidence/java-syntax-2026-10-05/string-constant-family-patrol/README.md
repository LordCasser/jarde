# 字符串与常量池家族巡查（2026-10-05 root，负结果）

## 探针

[fixture/ST.java](fixture/ST.java)（`--release 8`）：编译期常量字段（`static final String`/`int`，含内联）、非常量 static 字段、常量折叠混合拼接、`ldc2_w` long/double 常量、interned String `==` 比较、`hashCode()` 内联值比较。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **编译期常量字段级还原**：`static final String CS = "const-str";`、`static final int CI = 42;`（ConstantValue 还原）；非常量 `vs` 拆到 `static {}` 块赋值——与源结构完全一致；
- **常量折叠如实**：`CS + "/" + CI` 被 javac 预折叠为 `"const-str/42/"`，渲染如实呈现预折叠段 + 运行时段分离（`append("const-str/42/").append(ST.vs)`）；
- **`ldc2_w` long**（`123456789012345L`）与 **double 十六进制浮点拼写**（`0x1.921f9f01b866ep1d`）逐字精确；
- **interned 比较**（`arg0 == "lit" ?`）与 **hashCode 内联值**（`== 104`）如实（不做虚假去 switch 化——104 恰是 "h" 的哈希，但源码写的就是 hashCode 比较，忠实呈现优于臆测还原）。

**行为**：渲染源集 `javac` exit 0，`java -Xverify:all` 输出 `const-str/42/var-str/123456789012345/3.14159/yes/h` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。常量池家族（ConstantValue/ldc/ldc2_w/折叠/interned/hashCode）确认覆盖。
