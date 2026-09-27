# EM-10：调用表达式首片

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的四项库存测试，以及 `MethodInvokeVisitor`、`InsnGen` 源码哈希由 [replay.py](replay.py) 锁定。测试和实现只读；审阅到 `MethodInvokeVisitor.processInvoke` 如何由调用目标查找方法详情、泛型与重载信息，以及 `getCallClassFromInvoke` 如何区分实例接收者与静态声明类。`InsnGen.makeInvoke` 再按调用种类分别发射实例接收者、`super` 和静态 owner。Jarde 的对应路径位于 `jarde-java::build::Builder::call_expr`：receiver 与参数都从该调用点 SSA operands 渲染；静态调用 owner 来自该调用点的常量池目标，非静态调用从操作数中取真实 receiver。成员解析在 `jarde-jvm::members` 以 invocation kind 验证目标声明。

[固定的四类 Java 8 输入](input/)包含带副作用的实例 receiver、实例与静态方法的 `int/long/double/int` 参数、通过子类名调用继承的静态方法，以及 `catch (IOException)` 内的实例和静态调用。原始四类源码、JADX 四类输出和 Jarde 四类输出均以 `javac --release 8 -g:none` 完整重编，再由 `java -Xverify:all` 执行；三者输出均为 `195`。完整生成源文件保存在 [baseline](baseline/) 中，摘要包含 JADX 固定源哈希、输入和输出 SHA-256、原 class 哈希、Jarde CLI 哈希、编译结果及运行输出。

这条首片没有证明 EM-10 的缺口，因此没有新增实现 OpenSpec。JADX 对继承静态调用省略了可选的子类限定符（输出 `inherited(...)`）；Jarde 输出同样绑定到继承目标且完整重编、运行一致。覆盖仅限这组 Java 8 源码形状；其它 receiver 来源、异常区域中的局部变量合流、调用表达式副作用顺序、重载/泛型/可变参数与 Smali 输入仍需独立审计。一次更宽的试探把 catch 前局部变量带入 catch 后续表达式，Jarde 按现有边界拒绝了跨 fallback 区域绑定；该结果属于变量/控制流切片，不能作为调用恢复缺口归入 EM-10。

重放需要固定 JADX checkout 和已构建的 Jarde CLI：

```sh
cargo build -p jarde-cli
python3 openspec/evidence/java-syntax-2026-09-27/em10-invoke/replay.py \
  --jarde target/debug/jarde-cli \
  --out /tmp/em10-invoke-replay
```
