# clinit 匿名类单例巡查（2026-10-05 root，负结果——真实代码最常见策略/单例模式全过）

## 探针

[fixture/si.jar](fixture/si.jar)（源 [SI.java](fixture/SI.java)，`--release 8`）：**clinit 匿名**（无捕获字段位 `DOUBLE = new Op(){...}`、static 块内 `ADDONE = new Op(){...}`）、**注册表模式**（`Map<String,Op>` + put）、**工厂构造捕获匿名**（`makeCap(n)` 返回捕获 n 的 `SI$3(n)`）、blank final + 后声明字段初始化序。

## 结果：**健康，无缺口**（宿主+四伴生 quotes=0）

- **clinit 序= javac 发射序精确**：字段初始化（声明序 DOUBLE、REG）→ static 块体（ADDONE、put×2）→ 后声明字段（CAP）——blank final 与后声明字段初始化的交错序如实；
- 匿名伴生以**池形名**呈现（`new SI$1()`/`new SI$3(arg0)` 构造实参传递）+ 工厂方法返回捕获匿名（`return new SI$3(arg0)`）；
- 注册表消费（`(SI$Op) REG.get(...)` cast + apply 虚分派）恢复；
- 拼接 5 类型 `javac` exit 0 一次通过（**池形名 `SI$1` 作为类名合法**——`$` 是 Java 标识符合法字符）+ `-Xverify:all`；行为 `cap:7`/`10/6` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。clinit 匿名族（字段位/块内/工厂捕获/注册表）确认覆盖。
