# 静态/实例初始化块巡查（2026-10-05 root，负结果）

## 探针

[fixture/IB.java](fixture/IB.java)（`javac --release 8`）：多语句 `static {}` 块、两个实例初始化块（按源序）、`this(1)` 委派构造链、静态方法与 clinit 并存。

## 结果：**健康，无缺口**

jarde（合并态 `fff2fb3a`）：源码区 **quotes=0 / not-recovered=0**——

- `static {}` 呈现为 `static { IB.sc = 41; IB.sc = IB.sc + 1; }`，两条语句全恢复；
- 两个实例块**按源序并入** `IB(int)` 构造器（`this.ic = 10; this.ic = this.ic + 5;` 先于 `ic += arg1`）——与 JLS 的并入语义一致；
- `IB()` 的 `this(1)` 委派链保留；`more()` 与 `main` 正常。

**行为验证**：渲染源集 `javac` exit 0，`java -Xverify:all` 输出 `42/16/142` 与原 class 逐行一致（[results/](results/)）。

## jadx 对照（呈现取舍，非差距）

jadx 把首条 `sc = 41` **折进字段声明**（`static int sc = 41;`）并保留 `sc++` 在块内；jarde 两条语句都留在块内。两者编译产物与行为等价（本探针已双向验证），属**字段初始化推断的呈现取舍**——jarde 在其它片已做字段初始化呈现，此处未折叠到声明是保守但忠实的呈现，不立项。

## 处置

负结果归档，不立 spec。初始化块前沿（多语句、按序并入、委派链）确认覆盖。
