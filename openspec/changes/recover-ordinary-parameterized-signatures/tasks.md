## 1. 冻结对照与拒绝边界

- [x] 1.1 独立重放 `ordinary-parameterized-signatures/` 的 `-g`/`-g:none` 原 class、JADX、当前 Jarde：记录工具版本和完整哈希、单类/调用方 Java 8 编译、`-Xverify:all` 值及反射，并核对 root 已指出的 raw 返回编译失败位置；验证为同目录脚本可重复且结果表逐项一致。
- [x] 1.2 增补 verifier 可接受的擦除冲突、未呈现类变量、歧义内类/注解路径，以及参数化后可能改变正文或同类调用重载绑定的合法 Java 8 正反例；每条记录 `javap`、原类验证/运行和当前输出，不能以 verifier 不接受的字节充当拒绝依据。

## 2. 方法声明候选与安全发布

- [x] 2.1 复用 reader 解析/擦除结果，在类源码层对普通参数化 class、exact/any/extends/super 参数、嵌套参数及数组作有界 Java 8 拼写；不明 `$`/上下文类型变量整项拒绝。验证：结构化类型单元测试和 1.1/1.2 形状，查询引用顺序与 reader 定向测试不变。
- [x] 2.2 在方法头构造处用 descriptor 位置、同轮参数 slot/名字、`Exceptions`、varargs 及注解定位拼整项候选；对静态、实例和可证明的 `NoBody` abstract/interface/native 形状分别验证声明和重编反射，复杂合成构造器或注解路径有明确拒绝，不按文本替换。
- [x] 2.3 对 `Recovered` 方法使用同轮 Program/SSA 证明参数/返回源类型关系及受影响的正文调用；至少覆盖 1.1 的四个参数化 identity 方法和 raw 对照，错误 cast、参数写入、受泛型类型影响的重载调用必须拒绝。验证：正反 fixture 的完整类 `javac --release 8`、真实 Methodref/来源和独立方法报告不变。
- [x] 2.4 核对当前类对目标方法的调用绑定，不能证明的重载/推断变化整项拒绝；验证：1.2 的同类调用正反例原/Jarde `-Xverify:all` 结果与目标 descriptor 对照，拒绝保留物理调用和原头。
- [x] 2.5 只有类型、正文、调用、来源与费用全部成立才原子写回 `ClassSourceMethod` 的声明和文本；预算耗尽/取消不发布半头，essential/all 正文相同，`Recovered` 与 `NoBody` 的物理身份及独立报告不变。验证：低预算、取消、来源和属性位置定向测试。

## 3. 三方整类验收

- [x] 3.1 用重新构建的 CLI 对 1.1 四个有效普通参数化形状和 raw 对照分别生成完整类；原/JADX/Jarde 均 `javac --release 8` 重编、编译同一独立调用方、`java -Xverify:all` 逐行比较值与泛型反射，`-g`/`-g:none` 各自闭合；不以 Jarde 单类能编译代替调用方与反射。
- [x] 3.2 root 独立审读解析唯一性、逐位置擦除、类型拼写、同轮正文与调用证明、正反拒绝及来源；明确区分直接擦除冲突与嵌套类型存在性未证明的 List→Tree 变体，不将后者误报为可重编。复跑 reader/query、class-source、相邻泛型方法与增强 `for`、预算测试，完成 `cargo fmt --all -- --check`、适当严格 Clippy、corpus census/fingerprint、`git diff --check`、`openspec validate recover-ordinary-parameterized-signatures --strict` 后记录实测门槛与剩余独立债务。
