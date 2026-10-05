# this 泄漏/clone/长度循环巡查（2026-10-05 root，负结果+死指令数据点）

## 探针

[fixture/TL.java](fixture/TL.java)（`--release 8`）：构造器 **this 泄漏**双形（存静态字段、经参数——后者因 `inst` 为 static 而参数未真用）、**数组 clone**（`(int[]) data.clone()` covariant cast）、**长度边界 while 循环**、**自引用比较**、clone 独立性（改克隆不改原）。

## 结果：**健康（行为精确）+ 死指令数据点**

- **this 泄漏双形**全部正确（`TL.inst = this;` 在 ctor 内如实）；参数形有 1 条引注——javap 定位为 **`aload_1; pop` 死加载-弹出对**（javac 对未真用参数的 quirk；渲染语句完整正确，仅死指令对被诚实引注——**报告质量级**，非能力缺口）；
- **clone** 呈现 `(int[]) this.data.clone()`（covariant cast 精确）；**长度边界循环**（`local2 < local0.data.length`）与自引用比较恢复；
- 行为 `6/1/true` 逐行 IDENTICAL（clone 独立性=1：改克隆不改原——浅拷贝语义精确；this 泄漏 selfRef=true）。

## 处置

负结果归档，不立 spec；`aload_1; pop` 死指令对引注记为报告质量数据点（呈现完整，引注冗余）。
