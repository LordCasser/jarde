# 抛检查异常 SAM 与交叉类型 cast 巡查（2026-10-05 root，负结果）

## 探针

[fixture/XB.java](fixture/XB.java)（`--release 8`）：**抛检查异常的 SAM**（`int run() throws Exception`）+ lambda 实现与调用方 catch 链；**交叉类型 cast**（`(Serializable & Comparable<Serializable>) o`——javac 对每 bound 发 checkcast）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **抛 SAM**：接口方法 `public abstract int run() throws java.lang.Exception;` 的 throws 子句如实呈现；lambda 体（javac 常量折叠后 `() -> 1`——忠实于折叠产物）；调用方 catch 链恢复；
- **交叉类型 cast**：javac 对每个 bound 发独立 checkcast，jarde 呈现为**双 cast 链** `(Comparable) (Serializable) arg0`——物理事实逐字（等价：序贯窄化到第一 bound 的擦除类型）；
- 行为 `1/0` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。抛 SAM/交叉 cast 族确认覆盖。
