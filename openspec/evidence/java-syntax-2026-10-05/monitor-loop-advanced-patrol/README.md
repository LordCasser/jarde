# monitor 与循环进阶巡查（2026-10-05 root，负结果）

## 探针

[fixture/SY.java](fixture/SY.java)（`--release 8`）：类锁/对象锁 synchronized（含早退）、**嵌套同锁 monitor**、do-while + continue、for/while 混合 + `continue outer`/`break outer` 混合标签。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **synchronized 全形恢复**：类锁（`synchronized (SY.class)`）、对象锁（`synchronized (arg0)`）、早退分支、**嵌套同锁**（内层空体 + 外层 return——呈现与字节码几何一致且语义等价）；
- **do-while + continue**：continue 退化为 if/else（`if (i%2==0) {} else { s+=i; }`）——等价退化（同既证 continue 家族）；
- **混合标签**：`continue outer` 退化为内层 while 的 **`break`**（内层循环末尾恰是 continue 的语义目标——**等价**）；`break outer` 保留为 **`break loop`**（标签必需，正确保留）；`loop:` 标签命名符合既有 `loopN` 约定。

**行为**：渲染源集 `javac` exit 0、`java -Xverify:all` 输出 `6/1/7/9/4` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。monitor（含嵌套/早退）与循环进阶（do-while+continue、混合标签的等价退化边界）确认覆盖——`continue outer → break` 的等价条件（内层循环尾）与此前 multi-catch 巡查的结论一致（该退化仅在内层循环是外层体末条语句或 continue 位于外层体顶层时安全，jarde 判断正确）。
