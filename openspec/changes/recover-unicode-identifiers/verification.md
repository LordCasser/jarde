# 验证（change `recover-unicode-identifiers`）

## 实现（`crates/jarde-java/src/names.rs`）

`is_java_identifier` 的字符判定由 ASCII 表（`is_ascii_alphabetic`/`is_ascii_alphanumeric`）改为 JLS §3.8：
首字符为 Java 字母（`$`/`_` 同前），其余为 Java 字母或数字；关键字排除（`true`/`false`/`null`/`const`/`goto`/`_`）
逐字未动；零新依赖（`std` 的 `char::is_alphabetic`/`is_alphanumeric`）。

- **ASCII 快路径** `ascii_identifier`：`text.is_ascii()` 时逐字节走改动前的整张表，ASCII 判定与渲染因此可
  逐字节不动（下文语料扫描实测）。
- **别名生成器 `alias_for` 未动**：仍是 ASCII 表，别名一律 ASCII、修饰标记文本零变化；本片只缩窄「哪些名字需要别名」。
- **与 `Character.isJavaIdentifierStart/Part` 的关系**（判定函数文档逐条写明，非逐码点枚举）：
  - 两侧都收：Unicode 字母（`Alphabetic` ⊇ `isLetter` ∪ `Nl`）与数字（`is_alphanumeric` ⊇ `Nd`/`Nl`）；
  - 本片多收（超集侧）：`No` 数字（`²`/`①`）与具 `Alphabetic` 性质的组合标记出现在起始位；
  - 仍拒（Java 收）：`$` 之外的货币符号（`€`）、`_` 之外的连接标点、组合附加符、可忽略格式字符（U+200B）——
    `std` 无 Unicode 类别 API，放宽到「非空白非控制即收」会把 `。`/`·` 写进标识符位，正是该门要防的事；
    这些名字保持 `alias_for` + 原始名标记（与改动前逐字相同的处置）。

## 落点与双源一致（复核）

| 层 | 入口 | 判定 |
| --- | --- | --- |
| 字段/方法/参数/嵌套类**声明** | `src/class_source.rs::written_name`（真→池名，假→`alias_for`） | `is_java_identifier` |
| 局部/参数名（声明与使用） | `crates/jarde-java/src/names.rs` `NameTable::build_with_receiver` | 同一函数 |
| 成员引用、类字面量、嵌套名 | `crates/jarde-java/src/build.rs`（657/3658/3852/6718/8071/8163/26910） | 同一函数 |
| 字段证据、名字保留、类名门 | `field.rs:673`、`report.rs:5406`、`decode.rs:357` | 同一函数 |

判定只有一个所有者（`jarde_java` 重导出给 `jarde`），修复后声明与引用逐字一致（锚实测见下）。

## 锚实测 vs 预期（两条 javac 腿；`essential` + source map 入口）

fixture：`tests/fixtures/recover-unicode-identifiers/ut/`（真 javac 8 编译的 `UT.class`/`UT$内部类.class`）；
第二条腿在测试内由同一冻结源码 `javac --release 8 -encoding UTF-8` 现场编译。

| 锚 | 预期 | 实测 |
| --- | --- | --- |
| 声明层（`UT`，两腿） | `变量`/`描述`/`方法`/`内部类`/`名字` 本身 | `static int 变量;`、`static java.lang.String 描述;`、`static int 方法(int arg0)`、`static class 内部类 extends java.lang.Object`、`java.lang.String 名字;`；全文无 `__`、无 “is not a Java identifier” 标记、无 “not recovered” |
| 引用层（两腿） | 与声明同拼写（修复前已知正确） | `UT.描述`、`方法(21)`、`local1.名字`，且嵌套引用由池名 `UT$内部类` 变为源拼写 `内部类`（同一判定的引用侧） |
| 往返（两腿） | 剥离注释后 `javac --release 8 -encoding UTF-8` exit 0 | exit 0（断言编译成功，非静默） |
| 行为（两腿，`java -Xverify:all`） | `变量=1/42/中文`，与冻结 class 自身一致 | 逐字节 `变量=1/42/中文\n`，且与同 JVM 参数跑 fixture jar 的输出相等 |

对照（同一测试文件的另两条）：

| 对照 | 预期 | 实测 |
| --- | --- | --- |
| `Loc`（`-g`，真 javac 8；局部名来自 `LocalVariableTable`） | 字母名字如实呈现且声明/使用一致 | `static int 名字(int arg0)`、`int 数量 = arg0 + 1;`、`return 数量;` |
| `Loc.punct`（等价长替换 `数量`→`名。`，6 字节） | 表外字符仍走别名，且声明与使用同一别名 | `static int 名字(int arg0)`（方法名未受补丁影响）+ `int __ = arg0 + 1;`/`return __;`；全文不含 `。` |
| `package-info`（`pi.jar`） | 声明行名字路径未动（本片不新增合法性门） | 冻结文本（修前二进制渲染）逐字节相等，含 `interface package-info {`；与巡查 `results/jarde-packageinfo.txt` 的文本段人工比对一致 |

可证伪：把 `names.rs` 单独退回修前状态后，本片 4 个新测试 **3 个失败**（锚、往返、局部名对照），
ASCII 对照照旧通过——新检查确实能在旧行为上失败。

## ASCII 零回退（实测）

**语料扫描（before/after 逐字节 diff）**：语料 = `openspec/evidence/**` 的 **2721** 个类（每个独立 `.class`
与每个 `.jar` 内的每个 `.class` 条目；类名从 class 文件自己的 `this_class` 读出，不由路径猜）。两个二进制：
修前的 `816ee7b6` CLI 与本次 CLI，同参数逐类渲染（独立 class 用 `--policy single-class`、jar 用 `plain-jar`），
逐字节比文本：

| 项目 | 结果 |
| --- | --- |
| 非零退出 | 两侧同集合，各 **40**（既有拒绝类，stderr 也逐字节相同） |
| 可比文本 | 2681 份（其余为非零退出者，两侧同样） |
| 有变化的类 | **2** 个：`unicode-identifier-patrol/fixture/ut.jar` 的 `UT` 与 `UT$内部类`（即本片锚） |
| 其余 ASCII 语料 | **2679 份逐字节未动** |

`UT` 的 diff 内容（修前 → 修后，全文留档
`openspec/evidence/java-syntax-2026-10-05/unicode-identifier-patrol/results/jarde-UT.after.java`）：

```text
-    // jarde: the field's raw name `/u53D8/u91CF` is not a Java identifier; it is written as `__`
-    static int __;
+    static int 变量;
（`描述` 同形）
-    static int __(int arg0) {
-        // jarde: the member's raw name `/u65B9/u6CD5` is not a Java identifier; it is written as `__`
+    static int 方法(int arg0) {
-        UT$内部类 local1 = new UT$内部类();
+        内部类 local1 = new 内部类();
+    static class 内部类 extends java.lang.Object { … }        （成员类族折叠随之恢复）
```

**修前冻结文本**：`tests/fixtures/recover-unicode-identifiers/ascii/{F1,SB,RG,package-info}.jarde.java`
由修前二进制渲染，测试逐字节断言修后相等（F1/SB/RG 以单条目 jar 形状、`pi.jar` 原样）。

**单测**：`names.rs::the_identifier_table_is_javas_beside_a_byte_identical_ascii_half` 断言 ASCII 表逐例不变
（`int`/`while`/`true`/`null`/`_`/`1x`/`a-b`/`a.b`/空串仍拒；`a`/`$value_2`/`_x` 仍收）与表外 Unicode 仍拒。

**套件**：全量测试两轮（含既有 corpus 逐字节断言）结果见门禁节。

## 移动的既有 pin（如实记录，3 处）

1. `crates/jarde-java/src/decode.rs::modified_utf8_class_names_are_decoded_before_the_ascii_spelling_gate`
   → 更名 `…before_the_spelling_gate`：该测试钉的是「先解码 Modified UTF-8、后过标识符表」，表本身由本片扩宽，
   故 `𐐀`（Deseret 大写字母 U+10400，JLS 字母）由拒→收，断言改为收；并新增 emoji `😀`（合法 MUTF-8、
   **非**字母数字）负例保持拒——原测试的「可解码但表外」对照由此保留。
2. `crates/jarde-reader/src/classfile.rs` 的 fixture 人口普查 pin：`(687, 2932, …)` → `(691, 2943, …)`
   （本片 4 个 fixture 类 / 11 个 Code 体），逐字注释加在 pin 上方。
3. `tests/fixtures/corpus-fingerprint.json` 再生：**纯新增 10 项**（本片 fixture 与冻结文本；`README.md` 按指纹自身的 `md` 排除规则不入表），无既有条目变动。

## 门禁（本片最终状态）

| 命令 | 结果 |
| --- | --- |
| `cargo test --workspace --tests --locked --no-fail-fast` | exit 0；**309 targets / 3026 passed / 0 failed / 52 ignored** |
| `cargo fmt --all -- --check` | exit 0 |
| CI 逐字 clippy（`.github/workflows/ci.yml` 46–76，`--workspace --all-targets --all-features --locked … -D warnings`） | exit 0，无 warning |
| `openspec validate --all --strict` | 300 passed, 0 failed（`openspec validate recover-unicode-identifiers --strict` 单跑 valid） |
| `git diff --check` | exit 0 |
| 语料指纹 verify（再生后） | `p5_corpus_fingerprint` 6 测试全绿 |

基线对照：同一命令在上一片（`recover-comparable-argument-widening`）记录为 **308 targets / 3021 passed /
52 ignored**；本片 **+1 目标 / +5 通过 / 0 failed / 0 ignored**，差值恰为本片新增的 1 个单测 + 4 个集成测试。
**偏差如实记录**：派单引用的基线「315 targets / 3021 passed」目标数与本片同命令实测（309）不一致
（通过/忽略/失败三项一致：3021 + 5 = 3026、52/52、0/0），疑为调用形状不同（`--tests` vs CI 的
`--all-targets --all-features`）；以本片同命令实测为准，待 root 核对。

已知 flake：最早一轮全量（fixture 与两处 pin 尚未落齐时启动）中 `p3_two_exit_return` 失败，
**隔离复跑 7/7 通过**，此后两轮落齐后的全量（即上表两次）均通过 → 判为负载敏感 flake。该 target 的输入
全为 ASCII（一个 `openspec/evidence` 类 + `tests/fixtures/p3-two-exit-return` 的类），前者在语料扫描里
逐字节未动、后者在语料指纹里逐字节未动，故与本判定无关。既有四 flake（`p4_plugins`、
`p3_short_circuit_transfer_gateway`、`d3_artifact_binding`、`bulk_recovery_delivery`）两轮均未出现。

最早那一轮全量另有**两个预期失败**：`jarde-reader` 的 fixture 普查 pin 与 `p5_corpus_fingerprint`
——即本片新增 fixture 尚未更新两处 pin 所致（普查测得 `(691, 2943, …)`、指纹报未列文件，正是本片
要记录的那 4 类/10 项）。两处 pin 更新后 `p5_corpus_fingerprint` 6 测试与 `jarde-reader --lib` 全绿。

## 观察项（本片未处理，附证据）

1. **表外名字在成员引用位仍写池名**：探测 = 把锚 `UT.class` 的池项 `变量`（6 字节）等价长换成 `变。`
   （该序列出现 2 次：字段名与字符串常量 `变量=`，同长 6 字节，均被替换）。修后呈现：
   `static int __;`（声明走别名 + 原始名标记）而 `<clinit>` 里写 `UT.变。 = 1;`（成员引用按池名）。
   这是**引用路径自有的判断**（`build.rs` 的成员引用不查该表），改动前对 `变量` 也是同样的分叉
   （声明 `__` / 引用 `变量`）；本片只让**合法**名字两侧一致（spec 的 Requirement 范围），未改此路径，
   也不是本片引入。可作为后续窄片（成员引用位与声明同源判定）立项。
2. **类/包声明行不经该表**（proposal 的 root 追加关联的核对结论）：类声明名走 `simple_name` 原样拼接，
   `is_java_identifier` 只在泛型头投影把关；`package-info` 的连字符因此既不被收也不被拒，保持现状（本片
   不加声明行合法性门，冻结文本钉住不变）。
