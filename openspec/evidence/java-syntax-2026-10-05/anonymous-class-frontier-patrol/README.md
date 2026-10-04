# 匿名类前沿巡查（2026-10-05 root，负结果）

## 探针

[fixture/AN.java](fixture/AN.java)（`javac --release 8`）：匿名**子类**（`new Base(){…}` 覆写 `hi`）、匿名**接口实现**（`new Greeter(){…}`）、**带捕获**匿名接口（`new Greeter(){… n …}` 捕获 `final int`）三形。5.3 匿名内联链（四环）已覆盖内联呈现域；本巡查验证的是**未内联伴生类**（`AN$1/2/3` 单类政策渲染）与其宿主的组合健康。

## 结果：**健康，无缺口**

- **宿主 `AN`**（家族渲染）：源码区 quotes=0 / notrec=0；`mkBase/mkGreet/cap` 分别呈现 `new AN$1(this)` / `new AN$2(this)` / `new AN$3(this, arg1)`——池形伴生引用（与局部类巡查同判：已登记的池形伴生名声明债务域，jadx 同级——其 `new AN.1(this)` 点分形同样不可编译）。
- **三个伴生类**（`--class 'AN$N'` 单独渲染）：**全部恢复**（quotes=0）——
  - `AN$1`：`extends AN$Base` + `this$0` 捕获字段 + 覆写 `hi()` 体；
  - `AN$2`：`implements AN$Greeter` + `this$0` + `greet()` 体；
  - `AN$3`：`implements AN$Greeter` + **两个字段捕获**（`val$n` 先于 `this$0` 声明，构造器按 `super(); this.val$n=…; this.this$0=…` 呈现——与 javap 的实际写入序一致）+ `greet()` 体（StringBuilder 拼接如实展开）。
- jadx 对照（[results/jadx-AN.java](results/jadx-AN.java)）：宿主呈现同构（`new AN.1(this)` 点分 vs jarde 池形）。

## 巡查方法学事故（假零第 7 例，诚实登记）

首跑伴生类时我用了 `--policy single-class`，得到 JSON 错误 `environment_policy_snapshot_kind_mismatch`（jar 输入与该政策不匹配），输出文件**不含源码**——我的计数脚本却报了 `quotes=0 notrec=0`（对错误 JSON 计数自然为零）。**两层检查都失效**：自述头检查写在脚本里但首版脚本根本没查。修正：默认政策（jar 输入自动族折叠）+ 严格 `// jarde: presentation of` 头断言后才是上面的真实结果。教训并入 handoff 第 6 例族：**错误 JSON 输出与错误文本输出同样是假零源**，自述头断言必须是计数的**前置硬门**（非并行打印）。

## 处置

负结果归档，不立 spec。匿名类前沿（宿主引用 + 伴生类体 + 捕获字段序）确认健康；整类可编译性仍由已登记的池形伴生名债务统一解释，不重复立项。
