# clinit 抛出/Boolean 拆箱/复合循环条件巡查（2026-10-05 root）

## 健康面（负结果）

- **clinit 内抛出链**：`static { ready=false; VALUE=boot(); }`（boot 可能抛）完整渲染——clinit 序+常量字段赋值如实；
- **Boolean 拆箱条件（if 消费）**：`if (arg0.booleanValue())` 显式拆箱呈现（物理事实）；
- **装箱分支三元**：`b ? Boolean.TRUE : Boolean.FALSE`（拆箱条件+装箱分支）恢复；
- **do-while && 复合条件**、**while || 复合条件**逐字恢复（短路条件在循环位无碍——与三元位的拒绝对照鲜明）。

## 发现：布尔字面量分支三元（三元片第 5 数据点，已立项域）

`b ? true : false`（boolean 返回位）拒——**javap 证实与 `b ? 1 : 0` 发射完全相同的指令**（`iload_0; ifeq; iconst_1; goto; iconst_0; ireturn`）——int 返回位同指令恢复、boolean 返回位拒：差异纯在**布尔返回位的三元类型检查**（iconst join 需呈现为 boolean 字面量而检查器无法证明）。**jadx 解**：恒等折叠 `return b`（[results/jadx-CI.java](results/jadx-CI.java) 对拆箱形 `return bool.booleanValue()`）。

判别矩阵（三元片累计 5 数据点）：异型引用 / 上转型汇合 / 短路条件 / 字面量-vs-调用 / **布尔字面量分支（boolean 位）**。

## 处置

不新立；三元片锚维度 4 已补。
