# `recover-nested-type-source-spelling` 实现记录（-nn）

对应 OpenSpec change `recover-nested-type-source-spelling`（工作树基于主线 `48a44076`）。本目录
是它的实现取证与验收记录：出口取证、变体前后、恢复输出 SHA、corpus 扫描与门禁结果。

## 1. 出口取证（tasks 1.1）

**固定 V1 重放**：fixture SHA 与 `results/fixture-sha256.txt` 一致；主线基线（本变更前）两输入
输出与巡查时冻结的 `results/V1.jarde.java`（单类）和 `results/V1-jar.jarde.java`（fam.jar）逐字
相同——`variants/before/` 腿的 V1 两文件 SHA 即两者（`8025f2d3…`/`5c59259e…`）。缺陷复现：单类
输入 `$` 出现在参数类型与常量限定两处源码语法位；fam.jar 输入常量限定已被枚举折叠家族投影为
`V1.Op.MUL`，仅剩参数类型 `V1$Op`。

**类型引用拼写的出口**（呈现层类型名转换的实际分布）——本仓库没有单一物理函数，而是两条并行
的拼写缝，各自由常量池事实产出「源码形」名字：

1. **名字生产层（身份缝，判定为不可改）**：`build.rs::spell_reference`、`lambda.rs::
   type_of_base/source_name`（"the repository's one descriptor → Java type spelling"）、
   `class_source.rs::class_name`、四个模块局部 `source_name`（accessor/concat/lambda/bridge）。
   这些函数的产出同时是 **AST/证明层的语义身份**（lambda 工厂 owner 与 `Type::Reference` 名的
   等值比较、member-family 返回类型核对等），在名字生产层转换会让两侧不一致。
2. **文本写入层（呈现缝，本转换的落点）**：方法体一切类型位文本由 `emit.rs` 的 `Emitter`
   写出（新增 `put_type` 单一写入口）；类源码信封（成员声明参数/返回类型、字段类型）由
   `class_source.rs::spell_method/arguments/spell_field_declaration` 写出。两个写入层都持有
   「当前类」与 `InnerClasses` 行集。

**`X$N` 合成名共存边界**：匿名 `X$1` 与本地类 `X$1Local` 的 `$` 段以数字开头，不是 Java 标识符
——转换以「每个 `$` 段必须是 Java 标识符」为守卫，数字段一律逐字保留（CT-1 家族 `new C1$1(arg0)`
由既有测试锚定）。**顶层 `$` 名**（`Named$Top` 是一个真实顶层类名）：`$` 是合法标识符字符，
语法 split 无法区分——实现以 **JVMS §4.7.6 `InnerClasses` 行为证据门**：只有当前类自己的
`InnerClasses` 属性列为成员的名字才参与转换（`Named$Top` 由此逐字保留，既有 static-member 家族
测试锚定）。

## 2. 转换规则与落点（design 决策 1 的实现映射 + 取证修正）

实现过程中发现design 决策 1 的自嵌套分支需要一个更窄的边界，原因是**呈现文本与声明的可解析
一致性**：一个恢复文本只为「它自己折叠（fold）进来的成员声明」提供嵌套作用域；未折叠的成员
以池 `$` 名作为独立物理单元呈现，由这些单元组成的工程按 `$` 名解析。若把自嵌套引用一律改为
简单名，`H1.java` 引用 `H1$Greet` 变成 `Greet`，而平铺家族工程里没有任何文本声明 `Greet` ——
既有 9 个家族重编验收（user-hierarchy-widening、TWR、alias-field、enum 折叠家族孙类引用等）
全部破坏。最终规则：

- **跨类**（名字的顶层 owner ≠ 当前文本类的顶层 owner）→ 点分 `Outer.Inner`（多段全点分），
  以 `InnerClasses` 行为证据门。编译需该类以成员形式可用（编译类或嵌套源码）——与包级类同境，
  design 风险条目明示此为环境责任。
- **自嵌套**（owner 相同，含孙类）→ **保持池 `$` 名**，除非该成员被折叠进本文本：
  - **嵌套枚举折叠**（`nested_enum_family` 投影）：投影把根文本里该枚举名的 token（正文池形
    `V1$Op`、enum-switch 已点拼形 `V1.Op`、以及**成员声明的参数/返回类型 token**，后者按成员
    自身描述符 `LV1$Op;` 门控）统一重写为折叠拼写 `Op`（孙类 `Numbers.NumString` 形）。重写带
    source-map/FieldRef 物理锚点（正文）与派生 span 平移（声明），拒绝路径保持池拼写。
  - **已证静态成员家族** writer 既有的返回类型/创建点重写（`Leaf`）不变。

落点：转换核心 `crates/jarde-java/src/names.rs::nested_reference_spelling`（跨类点分 + 自嵌套
保持）与 `nested_member_reference_spelling`（InnerClasses 行门）；体层 `emit.rs::put_type` 覆盖
局部/for-each/资源/catch 声明、lambda 参数、cast、instanceof、new、类字面量、`Path` 限定、
`Type.super` 限定；信封层参数/返回/字段类型；枚举折叠投影的重写如上。`// @method`/`@declaration`
注释位不动。

**登记的后续切片**（非本变更范围，现状均为池 `$` 直拼）：类头 extends/implements 与 throws 子句
（拼写时点拿不到行集）；泛型 `Signature` 投影机械；嵌套枚举终态构造器体重发射；注解默认值中
的类字面量；**未折叠自嵌套成员的源码名拼写**（需要成员家族折叠机械扩展到接口/普通成员类——
即「呈现嵌套声明」能力，独立任务）。

## 3. 变体与前后（tasks 1.2 / 2.2）

变体 fixture 冻结于 `../fixture-variants/`（源码+class+jar+orig.out；`javac --release 8` 编译，
原类 `java -Xverify:all` 运行通过）。前后输出对（两输入）在 `variants/{before,after}/`，SHA 于
`variants/sha256.txt`，逐行差异于 `variants/diffs.txt`（全部为嵌套形态行，无其它文本变化）：

| 变体 | 覆盖 | before → after（源码语法位 `$` 计数） |
| --- | --- | --- |
| VN1 | 跨类嵌套（字段/参数/局部/new/instanceof/cast/静态限定） | `Other$Inner` → `Other.Inner`（8 → 1 行注释） |
| VN2 | 包名+深层：跨类 `p.A$B$C`；自嵌套 `p.VN2$Mid$Leaf` | 跨类 → `p.A.B.C`；自嵌套保持池名（7 → 2 行注释） |
| VN3 | 匿名合成名 + 自嵌套接口返回类型 | 两者均保持池名（`VN3$Op`、`new VN3$1`）逐字不变 |
| VN4 | 另一顶层类的静态嵌套类/接口类型位（参数/instanceof/局部/new/cast） | `Other$Box/Mark/Tagged` → `Other.Box/Mark/Tagged`（5 → 1 行注释） |

整类重编锚点（`tests/nested_type_source_spelling.rs` 内自动化，6 用例）：V1（fam.jar 折叠枚举）
`javac --release 8` 通过、`java -Xverify:all` 输出与 `fixture/orig.out` 逐字一致（参数 `Op`、
限定 `Op.MUL`）；VN1（跨类）与 VN4（跨类静态嵌套）重编+运行输出与原类一致；单类输入（折叠拒
绝）保持池拼写（负例用例）。

## 4. corpus 扫描（tasks 2.2）

`scan_corpus.py` 以两腿（主线 stash 前/本变更后）对 `tests/**` 全部 `.class` 逐类
`class-source --policy single-class`，结果见 `corpus/`。

## 5. 三方对照（tasks 3.2）

`three-way/jadx/`（固定 JADX dev 的 Java 输出与 `--release 8` 重编运行输出）与
`three-way/jarde/`（Jarde 恢复文本与其重编运行输出）。逐路径结论：V1/VN1/VN2/VN3/VN4 的
原 class 运行、JADX 路径运行、Jarde 恢复重编路径运行三者输出一致（V1 `20:-10`/`B:?`/`14:8`/
`3:3`，VN1 `242`，VN2 `12:11`，VN3 `42`，VN4 `144:true:t`）。JADX 自身对 VN4 拼写
`Other.Box`/`Other.Mark`/`Other.Tagged`，与本转换一致。V1 恢复文本 SHA `687acf5c…`，运行输出
SHA 与 `orig.out` 相同（`a8481f88…`）。

## 6. 门禁（tasks 3.1，实际命令与结果）

- `cargo test --workspace --tests --locked --no-fail-fast`：**287 个 test 目标全部 ok，0 失败，
  2863 个测试通过**（最终代码态复跑确认）。
- `cargo fmt --all -- --check`：通过（退出 0，无输出）。
- clippy（`.github/workflows/ci.yml` 实有 29 项 `-A` + `-D warnings`，
  `--workspace --all-targets --all-features --locked`）：通过（修一处
  `collapsible_str_replace` 后干净）。
- `openspec validate --all --strict`：**250 passed, 0 failed**。
- corpus 双腿扫描见 `corpus/`：452 个工件，448 逐字相同，4 差异（各一行，均为嵌套形态：
  `matrix.Outer$A` → `matrix.Outer.A`、`matrix.Outer$A$Generic` → `matrix.Outer.A.Generic`），
  7 个工件两腿同样非零退出（既有不可拼写 fixture，非本变更引入）。
- 已知 flake 家族未出现；本变更未触及 handoff.md 列出的任何 flake 测试。

任务书要求的 462 工件口径：本仓 `tests/**` 现有 452 个 `.class`（巡查 fixture 的变体类另存于
本目录 `../fixture-variants/`，已单独记录前后），扫描脚本 `scan_corpus.py` 固化于证据内可复跑。
