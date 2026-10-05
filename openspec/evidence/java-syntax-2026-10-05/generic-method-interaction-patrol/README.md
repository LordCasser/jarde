# 泛型方法级交互巡查（2026-10-05 root，宿主健康 + 混合保真度数据点）

## 宿主：健康（负结果）

[fixture/IC.java](fixture/IC.java)（`--release 8`）：方法级泛型交互全部恢复——菱形构造（`new IC$Box((Object) "s")` 擦除 cast 实参）、`(String) make().get()` 桥形 cast 消费、双参菱形、深泛型 `HashMap` 菱形、`(IC$Pair) nested()` 调用点上转型。宿主渲染源可编译可运行（与原行为一致由调用链保证）。

## 伴生：混合保真度数据点（既有债务的交互特征，非新缺口）

`IC$Box` 单独渲染：**类 Signature 投影成功**（`class IC$Box<T>`）+ **字段 Signature 投影成功**（`T v`），但 **ctor/方法 Signature 拒绝**（参数停留 raw `java.lang.Object`，`ordinary_generic_source_unproved`）→ 伴生体内 `this.v = arg1` 缺 `(T)` cast，拼接 `javac` exit 1（"Object 无法转换为 T"）。

**特征记录**：部分投影（头+字段投、ctor/方法不投）**比全 raw 更差**——全 raw（`Object v` 字段 + `Object` 参数）反而自洽可编译。这为 Signature 域的后续设计提供判据：投影要么成套（类+成员一致），要么对已投影字段的写入补呈现 cast。头明示 "not a compilable project"（响亮），故属**已登记的池形伴生债务 + Signature 域**的交互数据点，不新立。

## 处置

宿主负结果归档；混合保真度特征记入 Signature 域数据点（供后续 ordinary-parameterized-signatures 扩验片参考）。
