## Context

`src/facade.rs::prove_direct_parent_field_writes` 已按 `putfield` 的 BCI、CP 字段引用和选中直接父类字段表发证，但 `access_flags & (PUBLIC|STATIC)==PUBLIC` 排除 `protected` 与包可见实例字段。`crates/jarde-java/src/field.rs::verify` 只在该证书匹配 BCI、实际 SSA 接收者类型、owner/name/descriptor 后设置 `owner_cast`；source builder 已能打印 `((A)this).field`。DT-29 组合中 BCI 7/12 的 `aload_0` 接收者是 `B`，CP #11/#14 精确指向同包直接父类 `A`。固定 JADX 输出可编译，但对这两处写为 `this.field`；字段隐藏时不能直接照抄这种拼写。

## Goals / Non-Goals

**Goals:** 只在同包且同一选中直接父类唯一声明相符字段时，允许现有证书涵盖 `protected` 和包可见实例写入；保持物理 owner、一次写入、来源、预算及拒绝边界。隔离 fixture 的 `A` 和 `B` 各声明同名字段，setter 仅写 `A`，反射 runner 验证 `A` 的两个值为 true、`B` 的两个值为 false。

**Non-Goals:** private accessor 的新形态、跨包 protected 访问、static/final 字段、父类之外的祖先或接口、跨对象 `C`、泛型 `D`、根类 `run/bits`、任意字段读取或一般 Java 可见性推断。

## Decisions

1. 只扩展**已有** `ProvedSuperclassFieldWrite` 的发证条件：物理 `putfield` CP owner 必须是该方法声明类的选中直接父类；目标字段在选中父类中按 name 唯一，descriptor 完全一致。新准入为精确 `ACC_PROTECTED` 或包可见的普通实例字段；public 路径保持现状，private/static/final 及其他不在首片内的修饰符继续拒绝。
2. 父子类内部名在最后一个 `/` 前的包段必须逐字相同。这样显式 `((A) receiver).field` 对 `protected` 和包可见字段在 Java 8 源码中合法；跨包情形不通过这一证书。接收者仍由 field recovery 同一 SSA 值按声明类名核对，而非凭 `aload_0` 的字节模式猜测。首个正例只使用 `this`；该核对同样适用于可证明为 `B` 的其他接收者。
3. 不改变 AST/来源结构或 owner cast 打印规则。子类同名字段必须留在子类但不得吸收这两条写入；错 CP owner/name/descriptor、非直接父类、不同包、不匹配接收者均在原路径拒绝，并保留物理 BCI。
4. 正例以原/JADX/Jarde **完整类族源码** `javac --release 8 -g:none` 重编、`java -Xverify:all` 运行，比较反射取得的 `A`/`B` 四个字段值。固定 `TestFieldCast` 组合只验证 BCI 7/12 从拒绝变为受证写入；其余缺口继续列在 DT-29 账本，不因局部源码能打印就宣称整单元追平。

## Risks / Trade-offs

- 错把子类同名字段当成目标会产生可编译但不同义源码；沿用 CP owner 逐点证书和显式 cast，并在 runner 中同时检查 A/B 四个值。
- 跨包 `protected` 对强转后的父类接收者未必可从 Java 源码访问；首片仅限相同包，负例必须证明不同包不发证。
- 运行时类选择、预算或证据不完整时保留原有拒绝，不退化为仅按名字推断。
