# 静态嵌套访问与接口多态前沿巡查（2026-10-05 root，负结果）

## 探针

[fixture/IC.java](fixture/IC.java)（`javac --release 8`）：两层深静态嵌套的静态/实例成员访问（`Outer.Inner.stat`、`new Outer.Inner().instM()`）、接口参数多态（`viaIface(Op)`）、**lambda 入接口参数**（`x -> x*7` 传给非 SAM 感知的方法签名）。

## 结果：**健康，无缺口**

- 宿主 IC（家族渲染）quotes=0：**深层静态嵌套限定** `IC$Outer$Inner.stat + IC$Outer$Inner.statM()` 如实；`new IC$Outer$Inner()` 实例化+实例成员恢复；接口参数调用 `arg0.apply(3)` 恢复。
- **lambda 内联**：`(IC$Op) ((int arg0) -> arg0 * 7)` 内联进 `viaIface` 调用点（"lambda companion body inlined at invokedynamic@0" + 物理 helper 省略注释）——原生 `int` 参数的 lambda 呈现完整。
- 三个伴生类（`IC$Op` / `IC$Outer` / `IC$Outer$Inner`）单独渲染全部 quotes=0（硬自述头断言通过）。
- **拼接编译 + 行为**：四类型拼接 → `javac` exit 0 → `java -Xverify:all` 输出 `55/66/21` 与原 class **逐行一致**（[results/](results/)）。

## 处置

负结果归档，不立 spec。静态嵌套（两层深）、接口多态参数、原生参数 lambda 内联前沿确认覆盖。
