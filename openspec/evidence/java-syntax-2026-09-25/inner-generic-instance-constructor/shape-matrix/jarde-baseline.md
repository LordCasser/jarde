# Jarde 在双层成员矩阵上的实现前基线

使用本目录 [`fixture/`](fixture/) 的七个 Java 文件，`javac --release 8 -g:none` 编译后打成仅含这些 class 的 jar；以 2026-09-25 当前共享工作树构建 `jarde-cli`，对每个 class 运行 `class-source --format json --evidence all`。原始 jar 的 `Runner` 输出见 [`analysis.md`](analysis.md)，四个原始调用方都能对原始 `Outer` 类族独立重编。

| class | Jarde 方法/声明结果 | 证据边界 |
| --- | --- | --- |
| `UsePlainRaw` | `make` 无 `Signature`，但 body 未恢复；方法头为 `matrix.Outer$A` | 成员构造目标读取因外层 `A<T>` 有 class `Signature` 而拒绝 |
| `UsePlain` | `make` body 未恢复；`A<String>` 的方法 `Signature` 投影拒绝 | 除构造目标拒绝外，普通泛型方法头当前只接受受限返回形状 |
| `UseGenericObject` | 与 `UsePlain` 一样，未发出 `.new Generic<>(...)` | 泛型成员自己的类/构造器签名已有尾部对齐证明，但外层带签名被硬拒绝 |
| `UseGenericTyped` | body 未恢复；参数及返回的嵌套泛型方法头投影拒绝 | 最终输出为 `matrix.Outer$A$Generic` 擦除名，不是可重编的嵌套源类型 |
| `Outer$A` | 单类展示为 `Outer$A`；类签名 `<T:Ljava/lang/Object;>Ljava/lang/Object;` 未投影 | class-source 的泛型类头当前要求顶层名，拒绝 `$` |
| `Outer$A$Plain` | 单类展示为 `Outer$A$Plain`，构造声明含合成首参 `matrix.Outer$A` | 单类展示没有按外层所属关系组装嵌套声明 |
| `Outer$A$Generic` | 单类展示为 `Outer$A$Generic`；`V` 类签名和构造/访问器方法签名均拒绝 | `V` 未发布到源级类作用域，合成构造首参仍在通用声明里 |

所有八个 class-source 请求的 `execution.status` 均为 `complete`，这里的“未恢复”是明确的源码证明拒绝，不是读取失败。`Outer` 单类展示不包含 `A` 子声明。于是最小调用方修复与完整类族声明是两个不同的工作单元：前者复用已证成员构造站点，后者还需跨 class 装配和词法作用域，不能由本 change 的 caller-only 验收推导。

静态限定符另有一个独立于成员构造的可复现阻断：将 `package matrix; public class CallMark { public static int run(int v) { return Outer.A.mark(v); } }` 对本目录原始 `Outer` 编译，再请求其 class-source，Jarde 输出 `return matrix.Outer$A.mark(arg0);`。Java 8 源解析不了作为成员类型的 `matrix.Outer$A`。因此四个调用方的完整重编不仅需要修方法头和 `.new`，还需让同一已选 `A` 的静态调用限定符使用已证源类型路径；不能对所有 `$` 字符串做无证据替换。
