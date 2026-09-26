## 1. 基线与拒绝证据

- [x] 1.1 从冻结 `OuterSuperEffects` 复核原/JADX/Jarde 的局部成员类型、家族声明和 `Outer.super` 效果，记录 class 哈希、原/JADX Java 8 重编运行及 Jarde 当前编译错误；确认唯一失败位置与提案一致。
- [x] 1.2 准备合法 `$` 顶级类型、同名多定义、缺失/冲突成员路径及泛型成员类型对照；逐例验证不得凭名称拼接或部分路径投影，并记录预算/取消期望。

## 2. 声明类型源名

- [x] 2.1 复用同次已选成员目标的准确二进制类型和 `source_type_path`，只为无泛型实参且全路径一致的 `Type::Reference` 局部声明产生源名；单测证明同名顶级类、歧义目标和泛型路径不产生源名。
- [x] 2.2 在现有声明 AST 中与语义 `Type` 分别持有源名，统一用于普通声明、hoist 和 for 头的发射与 source-map 重放；定向测试检查赋值/合流语义类型未改变、正文和 BCI 区间未漂移。

## 3. 全类源码闭环

- [x] 3.1 用重建的 CLI 从冻结家族生成未编辑完整源码，`javac --release 8` 重编并以 `java -Xverify:all` 对照原/JADX/Jarde 的输出和效果次数；检查 `OuterSuperEffects.Member member = outer.new Member()` 与 `Outer.super` 同时存在。
- [x] 3.2 复跑 1.2 的拒绝对照、方法单独请求、默认/all 来源和紧预算/取消，确认未发布半成员名且物理方法身份/来源保留。

## 4. Root 独立验收

- [x] 4.1 Root 用独立 target 重建 CLI，重放原/JADX/Jarde 未编辑完整家族的 Java 8 编译与 `java -Xverify:all`，三方均输出 `12:123`、`fail:1`；成员类型源名和 `Outer.super` 同时存在。`jarde-java` 库测试 214/214、两项变更的集成测试合计 6/6、corpus fingerprint 5/5 通过；拒绝路径、来源、预算/取消在定向集成测试内通过，`cargo fmt --all -- --check`、`git diff --check` 和两个 OpenSpec strict validate 均通过。独立 Cargo target 验收后清理。
