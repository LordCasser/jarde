# this 捕获 lambda/methodref 巡查（2026-10-05 root，负结果+债务数据点）

## 探针

[fixture/TX.java](fixture/TX.java)（`--release 8`）：**this::method** 实例捕获方法引用、lambda 表达式体内 **this 引用**（表达式形/语句块形）、裸 this 字段对照。

## 结果：**健康（捕获链语义精确）+ 伴生名债务数据点**

- **this::compute 直接还原**（无 lambda 包装——`Sup local1 = this::compute;`，比 super:: 形更干净）+ `(Integer) get()` 拆箱恢复；
- **lambda 内 this 经 companion 转发**（`() -> this.companionViaLambda()`）+ **companion 体内 this.base 直读**（this 捕获链闭合——宿主与 companion 共享 this 无 this$0 字段，因为 companion 在同类内——物理事实如实）；
- 行为（伴生改名 stub 后）`10\n20/11/12/13` 逐行 IDENTICAL——四形捕获语义精确；
- **伴生名债务第 3 数据点**：直接拼接 `javac` exit 1——"符号 lambda$viaNested$1() 与 compiler-synthesized 符号冲突"（渲染源含 lambda → javac 重新合成同名 companion → 冲突）——与 Svc$Entry/LC$1L **同族**（池形伴生名债务），不新立。

## 处置

负结果归档；伴生名冲突数据点记入既有池形伴生名债务。
