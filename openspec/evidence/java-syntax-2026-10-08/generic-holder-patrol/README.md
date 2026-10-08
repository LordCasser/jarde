# 泛型字段与构造器巡查（2026-10-08）

基线 `085dfe39`（生产行为等于 ddfd05f8）；root 使用 `/tmp/jarde-main-baseline-cli`（在门控改动撤销后重新构建的主线二进制）。双腿：OpenJDK23.0.1 --release8 -g:none、Corretto1.8.0_432 -source8 -target8 -g:none。source、原始 class、完整 jarde 输出、剥离后的 Java 与 javac.log 均冻结。

## 实测

| 场景 | 两腿主线整类重编译 | 判断 |
| --- | --- | --- |
| Hold<T>：T字段、Hold(T) | 失败：Object不能赋给T | ctor header拒绝、field投影并存 |
| ObjectHold<T>：T字段、ctor(Object)，源码显式 unchecked `(T)` cast | 失败：Object不能赋给T | 不能仅通过修复Hold(T)来关闭 |
| ObjectSetter<T>：T字段、put(Object)，源码显式 unchecked `(T)` cast | 失败：Object不能赋给T | 独立field write投影缺口 |
| TypedSetter<T>：T字段、put(T) | 成功 | 正面边界，不能全部禁用field投影 |
| CrossSetter<T,U>：T字段、put(U)，源码 unchecked `(T)` cast | 失败：U不能赋给T | 相同Object擦除不能证明T/U可赋值 |

Hold原 class与JADX1.5.6完整输出均编译执行，外部driver（JADX只适配defpackage）逐字匹配：value=ok / classvars=1 / field=T / ctor=T。其他四族目前取证范围为源合法编译与Jarde完整文本重编译，不声称已完成JADX/行为验收。最初真实javac8比较缺少 -d 目标目录导致用法错误，修正目录后确认真实Object→T编译错误，保存的是修正后日志。

## 两个根因，分开立项

1. 字段投影：src/class_source.rs SameClassFieldUse/证明只检查读位（write的read_expressible=None默认通过），注释将“擦除值合法”推为“投影后源赋值合法”，方向错误。应首先对**每一个**写入值证明投影后的源码类型兼容；不成立则保持擦除字段与明确拒绝，而不是发布不可赋值的T。证明必须使用实际已发布的member source参数类型，不能把尚未获准投影的ctor Signature当成参数已是T；T/U与不同binder不能因擦除相同混同。TypedSetter是不可退化锚。
2. 构造器覆盖：project_method_signature仅将有method-local type params的<init>送到generic_constructor_declaration；Hold(T)只有class-scope T，落普通method分支后被<init>门拒绝。已有GenericConstructorCandidate只证明empty或参数转发prologue，不能承接字段赋值体。后续可以复用直线VoidBody参数槽证明与InitRecord，但必须连接parameter SSA→this.field与field/ctor Signature的同一type变量，不能直接去掉普通method的ctor gate。

下一步先写字段写位投影soundness change并验证安全回退和T参数正面形；之后独立写class-scope泛型ctor恢复change。两者都不混入当前functional-constructor-arguments片。审计建议由root独立确认，尚无代码实现与全量门禁结论。
