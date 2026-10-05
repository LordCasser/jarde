# 枚举抽象方法状态机巡查（2026-10-05 root，负结果+债务数据点）

## 探针

[fixture/EO.java](fixture/EO.java)（`--release 8`）：**抽象方法强制逐常量覆写**（状态机经典形——`ADD{apply} SUB{apply} IDENT{apply}; abstract apply`），javac 生成 EO$1/2/3 匿名子类 + 合成桥 ctor。

## 结果：**健康（语义精确）+ enum 变体债务数据点**

- **宿主全恢复**（quotes=0）：抽象声明（"no body: declared abstract" 诚实注记）、clinit 逐常量 `new EO$1("ADD", 0)`、合成桥 ctor `EO(String, int, EO$1)` 委派形、`$values()` 合成数组初始化、`values()` clone/valueOf——全部物理事实如实；
- **伴生 EO$1/2/3 全渲染**（apply 体各自完整）；
- 语义验证：源级等价形（常量体直接内联）行为 `8/2/5/SUB` 逐行 IDENTICAL——**分派、clinit 序、桥 ctor 语义精确**；
- **整类重编译失败**：枚举常量以静态字段+clinit new 形呈现（物理事实）而 javac 要求枚举常量声明在首部——**池形伴生名债务的 enum 变体**（与 Svc$Entry/LC$1L 同族：池形呈现无法回源级枚举常量形），登记债务数据点不另立项。

## 处置

负结果归档；enum 抽象方法族确认覆盖（与 5c13d781 常量体形互补——彼为源级可呈现形，此为抽象强制形）。
