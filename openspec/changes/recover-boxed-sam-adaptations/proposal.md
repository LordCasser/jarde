## Why

2026-09-26 巡查（自写 `Patrol` 场景，javac --release 8）发现：lambda/方法引用站点的 SAM 装箱适配未被 `lambda.rs` 的适配证明接受，携带此类站点的**整个方法**退为字节码引用：

- `Supplier<Integer> s = this::supply`（impl `()I`，instantiated `()Ljava/lang/Integer;`，erased `()Ljava/lang/Object;`）：拒绝「require a parameter or return conversion outside the proven identity, Object check, or Object upcast cases」。
- `Function<Integer, int[]> f = int[]::new`（impl 合成 `lambda$arrayCtor$0(I)[I`，instantiated `(Ljava/lang/Integer;)[I`）：同族拒绝，`arrayCtor` 方法整体引用。

javac 的实现方式是**不生成调用站点适配器**、把逐槽适配交给 `LambdaMetafactory` 的链接：擦除后的 SAM 类型、instantiated 类型与 impl 方法类型形成两段转换，基本类型 ↔ 对应包装类在链接期装箱/拆箱。`preserve-lambda-descriptor-adaptation`（9/9）已证明恒等、Object 检查、Object 上溯三类；本 change 把配对扩展到包装类/基本类型互转与数组构造器引用。`int[]::new` 的 impl 虽是 javac 生成的 `lambda$` 方法，名称本身不证明其 Code 只有数组分配。

jadx 1.5.6 同场景完整呈现箭头（`dispatch(this::supply, Patrol::staticRef)`）；jarde 当前整方法引用，是真实质量差距。

## What Changes

- lambda/方法引用站点的适配证明新增两类已证明形状：(a) 擦除 SAM → instantiated → impl 的同 arity 逐槽配对，包含「基本类型 ↔ 其唯一包装类」的装箱/拆箱（int/Integer、long/Long、float/Float、double/Double、byte/Byte、short/Short、char/Character、boolean/Boolean）；(b) 从同次 BSM 的精确 MethodHandle 追加按需成员候选，读取同一物理类的合成 impl Code，完整证明其只是从参数建立目标数组后返回，才准把使用点写成数组构造器引用。
- 箭头文本不变（`this::supply`、`int[]::new` 等）：适配在源码中不可见，改变的只是证明接受的形状集。
- 类级源码在证明合成数组 helper 没有其它未投影使用后才省略它的物理声明，避免 javac 重新生成同名 helper 时发生符号冲突；方法/JSON 报告继续保留物理成员。
- arity 不一致、非互转类型对（如 `String`↔`int`）、静态性不匹配、impl 读取不到时保持既有拒绝。
- 自写 fixture 三方对照与重编译执行（通过 SAM 调用观察装箱适配的值与异常）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增 SAM 适配的包装类/基本类型配对与数组构造器引用要求。

## Impact

实现集中在 `crates/jarde-java/src/lambda.rs`（两段适配配对判定）、`src/facade.rs` 的既有按需 callee 候选/类装配接缝及 `build.rs` 的原箭头发射器。捕获适配、创建时求值路径不变；不新增公开 pass、crate 或生产依赖。

非目标：泛型签名投影（`generic_call_binding_unproved` 门）、捕获值的新适配、原始类型特化（`ToIntFunction` 等非 metafactory 路径）、bridge/菱形适配。
