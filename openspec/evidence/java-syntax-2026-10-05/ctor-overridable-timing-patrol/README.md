# 构造器调用可覆写方法的初始化时序巡查（2026-10-06 root，负结果）

## 探针

[fixture/CO.java](fixture/CO.java)（`--release 8`）：基类 ctor 调 `hook()`（template-method 陷阱——`B.b` 在 `A()` 执行期间仍为 0）、子类字段初始化、实例初始化块+显式 super 后追加语句、宿主渲染内联三伴生。

## 结果：**健康，无缺口**（全类 quotes=0）

- **初始化次序保真**：`B` ctor 呈现 `super(); this.b = 10;`——`hook()` 求值时 `b=0` 的陷阱逐字节保留；`C` 的 `{c=5}` 并入 ctor 后 `c=c+1` 次序正确；
- 宿主渲染内联三伴生（`static class B/C` + `static abstract class A`，A 无 Code 诚实注记）；伴生单类渲染同构；
- 剥离往返编译 exit 0、`-Xverify:all` 行为 `1/10/7/6` 逐行 IDENTICAL（若字段初始化被重排到 super 前则 `a` 会变 11——判别成立）。

## 处置

负结果归档，不立 spec。ctor×虚方法×初始化次序域确认覆盖（与 #monitor-timing 巡查互补：本域次序保真）。
