# DT-13：嵌套枚举与 `enum implements I` 对照

## 结论

本次把 DT-13 拆成两个独立子形态。`enum implements I` 的声明头在冻结样例上与 JADX 对齐；嵌套枚举存在可复现的源 API 差距：Jarde 单类源码入口将成员枚举输出为 `$` 名顶层类型，丢失词法嵌套关系。全部生成的物理类文本集合自身可编译；改用原始消费者的 `NestedShape.Major` / `NestedShape.Major.Minor` API 时，Java 8 编译失败。只用展开的二进制名 `NestedShape$Major` / `NestedShape$Major$Minor` 编译、`-Xverify:all` 运行则成功，说明差距是类级源码装配与成员声明投影，不是枚举常量运行逻辑。

结果仅覆盖本文冻结的简单形状，不宣称 `TestInnerEnums` 全部能力已追平：其枚举构造实参（byte 与另一枚举）另属 DT-11；`TestEnumsInterface` 还含每个常量独立匿名子类和覆盖方法，属于 DT-12，不在本次接口声明验收内。

## 固定基线与 JADX 断言质量

- JADX checkout：`/Users/lordcasser/workspace/testzone/jadx`，固定 HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。
- 对照测试：`jadx-core/src/test/java/jadx/tests/integration/enums/TestInnerEnums.java` 与 `TestEnumsInterface.java`。
- `TestInnerEnums` 有执行 `check()` 的运行时断言，但其源码断言只检查 `ONE((byte) 1, NumString.ONE)` 与 `ONE("one")` 各出现一次；不能证明完整源码可重编或嵌套 API 行为。
- `TestEnumsInterface` 的断言检查 `PLUS` / `MINUS` 常量体文字，但这一叠加了 DT-12 匿名体恢复。因此这里单独对 `implements Marker` 做无匿名体枚举正例。
- JADX 源码内层类由 `ClassGen.addInnerClass` 递归生成；类声明从 class facts 输出 interface list。`EnumVisitor` 做 enum `<clinit>` 恢复、常量识别和 access flags 修正。`TestEnums3` 不是本 DT-13 正例。

## 冻结 fixture 与重放

冻结输入位于本目录：`NestedShape.java`、`InterfaceShape.java`、`PlainShape.java`、`PlainImpl.java`、`Runner.java`、`JardeFlatRunner.java`。重放脚本 `replay.py` 用 `TemporaryDirectory` 构造 class/jar、JADX 输出和 Rust `CARGO_TARGET_DIR`，退出自动清理。它使用 `javac --release 8 -g:none` 生成原始 class；对原始、JADX 和 Jarde 完整输出均作 Java 8 编译及 `java -Xverify:all`。Jarde 通过固定 jar 的 `class-source --policy plain-jar --release 8` 对每个物理类读取完整 class-source 输出，再汇总成完整源码集合。CLI 可以通过 `JARDE_CLI` 指向预建 CLI，否则 cargo target 留在临时目录。

在仓库根目录执行：

```sh
python3 openspec/evidence/java-syntax-2026-09-27/dt13-enum-shapes/replay.py
```

脚本重放时检查 JADX HEAD 与固定 SHA 相同，并以硬断言要求：original/JADX/展开名 Jarde 完整源码按 Java 8 编译及 `-Xverify:all` 运行成功且输出一致；原嵌套 API 的 Jarde baseline 必须精确报出四个 `NestedShape.Major` class/variable `cannot find symbol`；接口 enum 输出必须保留 `implements Marker`，plain enum 保持无接口行为。结果文件 `results.json` 保存工具版本、冻结输入与 class SHA-256、源码编译/运行状态、断言结果和稳定化后的精确 Jarde API 编译诊断。`original-javap.txt` 保存原始 class 的 `javap -v -p` 输出；重放会将 class 文件路径替换为 `<TMP>`，将 `Last modified` 日期归一化，保留 class SHA 和结构内容。Jarde 每个 class-source 请求仅记录成功状态，不保存含时长/临时位置等易变 stderr 的 hash 或长度。生成源码输出采用最小物理类结果保留以供复核，没有保存临时 class 或 cargo target。

本次环境：`javac 23.0.1`，通过 `--release 8` 生成 Java 8 class；`java -Xverify:all` 负责验证执行时的 class 字节码。输入 SHA 和产物 SHA 均见 `results.json`。

## 观察到的源码与 classfile 形状

原始源码 API：

```java
public final class NestedShape {
    public enum Major {
        FIRST, SECOND;
        public enum Minor { LEFT, RIGHT }
    }
}
```

`javap -v -p` 显示实体 class 为 `NestedShape.class`、`NestedShape$Major.class` 与 `NestedShape$Major$Minor.class`。`Major.class` 的 `InnerClasses` 行将 `Major` 指向 `NestedShape`；`Minor.class` 的行将 `Minor` 指向 `NestedShape$Major`：

```text
public static final #...= ... // Major=class dt13/NestedShape$Major of class dt13/NestedShape
public static final #...= ... // Minor=class dt13/NestedShape$Major$Minor of class dt13/NestedShape$Major
```

两个 enum class 都具有 `ACC_PUBLIC, ACC_FINAL, ACC_SUPER, ACC_ENUM`；它们是普通 Java enum 常量与 `$VALUES` 的标准 Java 8 构造形状。

JADX 输出在 `NestedShape` 内递归写 `public enum Major` 和 `public enum Minor`，并保持 `observe()` 中的 `Major.FIRST` 与 `Major.Minor.LEFT`。它将本例的完整输出按原 Runner API 重编、运行。

Jarde 当前分别输出：

```java
// NestedShape.java 中
return new java.lang.StringBuilder()
    .append((java.lang.Object) dt13.NestedShape$Major.FIRST)
    .append(":")
    .append((java.lang.Object) dt13.NestedShape$Major$Minor.LEFT)
    .toString();
```

而 enum 物理类各自仍为顶层声明：

```java
public enum NestedShape$Major { FIRST, SECOND; }
public enum NestedShape$Major$Minor { LEFT, RIGHT; }
```

完整的实际文本见 `jarde-dt13_NestedShape.java.txt`、`jarde-dt13_NestedShape_Major.java.txt` 和 `jarde-dt13_NestedShape_Major_Minor.java.txt`。

## 三方重编与运行结果

共同 Runner 打印：

```text
nested=FIRST:LEFT:FIRST:LEFT
interface=FIRST:true
plain=FIRST:false
class=true
```

| 源码集合与消费方式 | Java 8 编译 | `-Xverify:all` 运行 |
| --- | --- | --- |
| 原始 fixture + 原 Runner | 通过 | 上述输出 |
| JADX 完整源码 + 原 Runner | 通过 | 与原始输出逐字节一致 |
| Jarde 的所有物理类源码集合 | 通过 | 不含调用者，仅证明展开名集合相互自洽 |
| Jarde 完整源码 + 原 Runner（嵌套 API） | 失败 | 未运行。4 个 `cannot find symbol`：`NestedShape.Major` 类/变量各 2 次 |
| Jarde 完整源码 + `JardeFlatRunner`（`$` 顶层 API） | 通过 | 与原始输出逐字节一致 |

接口正例的 Jarde 声明是 `public enum InterfaceShape implements dt13.Marker`，原始与 JADX 输出也保留 `implements Marker`；运行时 `InterfaceShape.FIRST instanceof Marker` 为 `true`。负边界同时检查无接口的 `PlainShape` 保持 `false`，且普通类 `PlainImpl implements Marker` 保持 `true`，避免把接口事实错绑到 enum 或类声明种类。

## Jarde 当前架构入口与复用边界

当前公开 `class-source` 流程在 `src/facade.rs::Engine::class_source_with_evidence` 进入 `prepare_physical_class_source`，并以 `class_source::class_declaration_with_types` 按当前物理 class facts 写一个 class 的头部。该 writer 能从 enum flags 输出 `enum`，也能按其接口列表输出 `implements`；上面的接口正例证明了此子形态。

枚举常量组由同文件调用 `enum_constants::prove_group`、随后 `class_source::prepare_enum_constant_source_projection` 投影。它对单一枚举自身的字段、构造器与初始化证据负责，不声明这个枚举属于哪个词法 outer class。

现有具名成员类路径从 `member_inner::scan_family_root` 进入 `prepare_class_source_member_family`，再调用 `project_class_source_member_family`。`ClassSourceMemberFamily` 是根报告中的来源关系、子报告、调用/捕获事实与可选源文件投影的报告载体；当前候选和投影针对具名成员类族的既定构造/捕获场景，不足以直接证明 enum child 可以插入父类声明列表。未来实现可复用其中的已选定义/同次 InnerClasses 关系、独立 child report 与根级投影边界，但必须扩展并专门证明 enum 的 lexical owner、递归父子行、访问标志及完整 enum 组证书。

`class-source` 单个结果明确不承诺可编译项目。因此这里把所有单类报告作为临时完整源码集合重编，不把 CLI 源码输出包装文本宣称成工程级恢复。更根本的差距仍成立：汇总后只有展开 `$` 顶层名能工作，原始嵌套 API 不存在。

## 范围建议

已建立独立窄 OpenSpec `recover-proved-nested-enum-source`：只讨论在父类/enum 词法位置输出已完整证明的嵌套 enum 声明，覆盖 fixture 中两级 `NestedShape.Major.Minor` 关系；维持整组拒绝和物理类报告，不处理任意成员类。

`enum implements I` 无需另开实现任务；本样例无差距。TestEnumsInterface 里的匿名常量体继续归 DT-12。带构造参数的嵌套 enum 不在这份窄切片中，它本身受 DT-11 证据约束；首个实现可限制为当前无参数/普通常量组的形状，或明确依赖已通过的现有 enum proof，但不得据此声称 TestInnerEnums 已追平。
