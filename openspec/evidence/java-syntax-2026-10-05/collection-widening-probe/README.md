# Collection 扩宽独立探针（2026-10-05 root，负结果——第 5 表否定）

## 探针

[fixture/CL.java](fixture/CL.java)（`--release 8`）：`List.retainAll(Set)`/`containsAll(Set)`（java.util 实参到 Collection 形参位）、`Collections.disjoint` 静态泛型、`size(T extends Collection<?>)` 泛型方法擦除调用点（`size(new ArrayList<>(...))`——ArrayList presents Collection）。

## 结果：**独立恢复，无 Collection 扩宽族**（quotes=0）

四形全部恢复；泛型方法擦除调用点甚至呈现显式 `(java.util.Collection)` cast（pool 物理事实如实）。**结论**：[诊断普查](../diagnosis-census/README.md) 的第 5 表线索否定——OB 里 `EnumSet presents java.util.Collection` 行是 Enum 锚同方法的**级联伴随**（与其主行同现），非独立失败面。四表扩宽族维持封闭（CharSequence/Comparable/Serializable/Enum）。

## 附注（探针工程）

javac8 diamond+泛型方法在实参位置的嵌套推断限制（JDK-8028143 族）迫使 main 用局部变量——源级编译问题与 jarde 无关，已绕开。

## 处置

enum 片的 Collection 级联注记改为"非独立族"；census 第 5 表线索关闭。
