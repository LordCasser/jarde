## `p3-deferred-value-order` fixture

本次只固定一份正面 Java 8 class：`DeferredValueOrder.class`。`DeferredEffects.java`、
`OrderValue.java` 与 `DeferredValueOrderRunner.java` 是 source-only helper/runner；源码
`DeferredValueOrder.java` 是未插入独立调用的可重编译输入。真正的执行 oracle 是由
`evidence/java-syntax-2026-09-22/deferred-evaluation/fixture/patch.py` 从这份源码编译后，
对完整 `Code` 属性做精确字节替换得到的 patched class，不能用未补丁 class 替代。

基础直线 producer/consumer 覆盖调用结果、静态字段、数组元素、`arraylength`、普通 cast、
`new` 构造、实例字段读取、int/long 除法和取余，以及 `newarray`、`anewarray`、
`multianewarray`。同一 class 还保留嵌套 inline、分支内直线和 null 测试前缀作为合法结构
对照。独立 `mark()` 在 14 个直线方法中均插在 producer/检查完成后、原 return 前；
`ArithmeticException`、`NegativeArraySizeException`、`NullPointerException` 和
`ClassCastException` 的失败路径因此能证明检查顺序没有被后续效果替代。runner 的每个
案例都独立 reset `trace`、字段、数组、除数和 holder；`mode=0/1/2` 分别覆盖正常路径、
producer/构造器失败以及 `mark()` 失败，不让前一个案例污染后一个异常轨迹。

重放命令：

```text
python3 openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/fixture/patch.py \
  --work /tmp/jarde-deferred-value-order-fixture
```

脚本只匹配完整 `Code` 属性，按 `javap` 的本次常量池引用构造 `invokestatic
DeferredEffects.mark:()V`，每个目标方法恰好一次；没有通用 class parser。脚本会用
`java -Xverify:all` 执行 patched class，并写出 patch manifest 和 BCI 反汇编。

## 冻结事实

未补丁源码编译出的 `DeferredValueOrder.class` 为 1296 bytes、SHA-256
`5fd915aa1b803e418c020c06b1db2ed09dd16e30177a7cc32017791e3b130898`。14 次 Code patch
之后，永久 fixture 为 major version 52、1338 bytes、19 个 `Code` 方法（构造器加 18
个正面方法），SHA-256 为
`44eaaf23c354566f25ba429e49f7d34a73a114c7de35909360b705dcc63c3d04`。完整 patched runner
原交接输出 77 行；root 随后补入 null 数组读取与空数组越界各 3 个 mode，当前 runner
输出 83 行：15 个主要方法各执行 3 个 mode 共 45 行，负长度/零除/零余/null receiver/
数组读取异常共 33 行，另有 5 行嵌套、分支和测试前缀。只增加 source-only runner 输入，
永久 class 的字节、hash 与 Code 数不变。

14 个被插入的方法及原始/补丁字节保存在 `fixture/patch-manifest.json`：`call`、`field`、
`array`、`instanceField`、`arrayLength`、`newArray`、`anewArray`、`multiArray`、`divInt`、
`remInt`、`divLong`、`remLong`、`cast`、`direct`。每个 patch 保持原生产/检查在 `mark()`
之前，构造器自身和 getfield/除法/数组长度检查也仍先于后续效果。

## 三方基线

以下是原 77 行交接基线，文件保存在 `deferred-evaluation/fixture/`：

- `original.txt` 是 patched class 经 `java -Xverify:all` 的 77 行 oracle；
- `jadx.java.txt` 未经手工修改，JADX 完整源码和 source-only helper/runner 通过
  `javac --release 8`，执行 77 行且与 `original.txt` 逐行一致；
- 当前旧 jarde CLI 的完整 class-source 输出在 `jarde.java.txt`，完整重编译成功并执行
  77 行，但与 patched oracle 有 42 行差异（0 quote）；这正是错序值已经被正常 Java 文本
  发布的红基线，不把它当作通过；
- `source-javac.log`、`source-javap.txt`、`patched-javap.txt`、`jarde-javac.log`、
  `jarde-report.txt`、`jadx-javac.log` 与 `patch-manifest.json` 保存命令和物理证据。

原审计的 18 项 call/field/array/cast 与 3 项 construction 证据继续保留在
`../` 的 deferred-evaluation 和 construction-consumers 目录；本 fixture 把已确认的
`arraylength`、除法/取余及三种既有数组创建形状纳入同一共享位置回归，不新增多个永久 class。

## 边界

root 在 `fixture/root/` 独立重放精确补丁，确认源码编译/patch 的字节与冻结 class 完全
相同。新增的 83 行原 class 与 JADX 完整输出一致；CLI
`c050b502ba3fc23f33937ac607b1da7b51447e02e1a6d41b1d060c2b93b5b8d9` 的完整 jarde 输出
仍 0 quote、javac 成功，但有 48 行差异。null 数组和空数组的原始异常均发生在 mark 之前；
错误重排会改变 trace，并让 mark 的异常抢先。Cargo RED 尚未运行，不将 Java 对照代替 Rust 测试。

本轮只冻结可由完整 Code patch 验证的直线 producer/consumer 和合法结构对照；跨 region、
guard、失效局部、重复消费或无法证明词法声明的拒绝输入仍由实现阶段的内存变体和 root
独立审计负责，不伪造无效 JVM class，也不在 fixture 中声明这些边界已经关闭。
