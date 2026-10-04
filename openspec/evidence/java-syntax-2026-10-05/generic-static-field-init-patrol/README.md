# 泛型静态字段初始化巡查（2026-10-05 root，已证缺口）

## 结论

静态泛型字段的初始化表达式渲染为**不可编译的损坏文本**：`static Hold<String> f1 = new Hold<>("a")` → `static Hold f1 = new MN$Holdava.lang.Object) "a");`（丢括号/类型实参，类名纠缠）。**javac exit 1**；jadx 完整恢复同形——有解缺口。实例字段（f3）退化为构造器赋值，保守但合法，不受本缺口影响。

## 判别

菱形与显式类型实参同样坏 → 与 diamond 无关；触发条件是“静态字段 + 泛型 Signature 投影被拒（`field_generic_body_unproved`）+ 初始化含泛型类构造调用”。伴生成员类 `Hold<T>` 自身的 `T v` 投影正常（“projected after descriptor erasure”注释可见）。

## 处置

新登记缺口（呈现缺陷级——uncompilable 文本，比响亮拒绝严重）。候选落点：字段 Signature 投影拒绝（`class_source.rs` 的 `field_generic_body_unproved`）后的**初始化表达式回退路径**——当前把构造调用的接收者/实参拼坏。待独立立项（窄片：先把损坏文本修为响亮拒绝或裸类型正确形）。
