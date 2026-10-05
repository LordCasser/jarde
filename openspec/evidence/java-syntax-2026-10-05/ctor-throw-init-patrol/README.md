# 构造器抛异常与初始化器交互巡查（2026-10-05 root）

## 结果

- **ctor 域全部忠实**（负结果）：静态初始化器字段级还原 + 实例初始化器并入 ctor（`this.ic = init("ic", 2)` 在 `super()` 后、用户代码前——JLS 序）；**提前 `throw`**（`if(x<0) throw …` + else 分支携带正常路径与 return）；**委派构造器**（`this(-1)` + 委派后 unreachable `println("never")` 如实保留）；
- **行为**：手工补丢弃语句后渲染源集 `javac` exit 0、`-Xverify:all` 输出 `sc/ic/ctor ok:5/ic/caught:neg:-1` 与原 class **逐行一致**；
- **main 的 6 引注**：两处丢弃分配（`new CE(5);`/`new CE();`）——**第 5 窄片（丢弃分配）的 try-块变体**（BCI 0/3/9/12 同族诊断），已作为追加锚写入该片 proposal；ctor 抛异常 + catch 捕获路径的交互由该片的 CE 锚覆盖。

## 处置

不新立（丢弃分配域）；CE 追加锚已登记。
