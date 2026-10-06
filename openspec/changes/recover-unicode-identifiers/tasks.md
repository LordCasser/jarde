# 任务（change `recover-unicode-identifiers`）

## 1. 取证与复核（实现前）

- [x] 1.0 复核落点：判定函数是否单一、声明行与引用行是否同源。
      → 实测：判定只有一个所有者 `crates/jarde-java/src/names.rs::is_java_identifier`（`jarde` crate 经
      `jarde_java` 重导出）。**声明层** `src/class_source.rs::written_name`（`is_java_identifier` 判真则用池名、
      假则 `alias_for`）供给字段/方法/参数/嵌套类声明，仓库内 13 处调用点；**引用层** `crates/jarde-java/src/names.rs`
      的 `NameTable`（局部名）、`build.rs`（成员引用、类字面量、嵌套名）、`field.rs`、`report.rs`、`decode.rs`
      全部走同一函数。修复后 `UT` 实测声明与引用逐字一致（第 3 节）。
- [x] 1.1 基线扫描（修前二进制）：`openspec/evidence/**` 全部 **2721** 个类（独立 `.class` 用
      `--policy single-class`，jar 用 `plain-jar`，类名从 class 文件 `this_class` 读出）逐类渲染，作为零回退对照。
      → 实测：修前 40 个非零退出（同一集合修后不变）；2681 份文本留档。
- [x] 1.2 边界核对（proposal 的 root 追加关联）：类/包声明行是否同源。
      → 实测：**不同源**——类声明名走 `simple_name`（原样），`is_java_identifier` 只在泛型头投影
      （`class_generic_source_unproved`）上把关；包声明走 `package_name`，无判定。故 `package-info` 的连字符
      不经本函数，本片**不新增声明行合法性门**（按 proposal 记录，保持现状拒/注记）。

## 2. 实现

- [x] 2.1 `is_java_identifier` 的字符判定从 ASCII 表扩为 JLS §3.8 语义（字母 + 数字 + `$`/`_`，关键字与
      `true`/`false`/`null`/`const`/`goto`/`_` 的排除逐字不变），由 `std` 的 Unicode 性质实现，零新依赖。
- [x] 2.2 ASCII 快路径：`text.is_ascii()` 先走 `ascii_identifier`（逐字节 `[A-Za-z_$][A-Za-z0-9_$]*`，
      即改动前的整张表），保证 ASCII 判定与渲染逐字节可保持不变。
- [x] 2.3 别名生成器 `alias_for` **不动**：它仍是 ASCII 表（别名一律 ASCII、标记文本零变化）；本片只缩窄
      「哪些名字需要别名」，不改变别名的字节。

## 3. 对照测试与 fixture 冻结

- [x] 3.1 锚 fixture：`tests/fixtures/recover-unicode-identifiers/ut/{UT.java,UT.class,UT$内部类.class}`
      —— 巡查 `UT.java` 逐字复制，由**真 javac 8**（Corretto 1.8.0_432，`javac -encoding UTF-8`）编译；
      第二条腿（工具链 `javac --release 8 -encoding UTF-8`）在测试内由同一冻结源码现场编译。
- [x] 3.2 局部名对照：`loc/Loc.java`（`-g` 编译，`LocalVariableTable` 里中文局部名 `数量`、方法名 `名字`）与
      `loc/Loc.punct.class`（等价长替换 `数量`→`名。`，6 字节；README 记补丁配方）。
- [x] 3.3 ASCII 修前冻结：`ascii/{F1,SB,RG,package-info}.jarde.java` —— 由**修前二进制**（基线 `816ee7b6`）
      对既有 patrol fixture 渲染，测试断言修后逐字节相等（含 `interface package-info {` 的声明行边界）。
- [x] 3.4 单测对照：`names.rs` 新增 `the_identifier_table_is_javas_beside_a_byte_identical_ascii_half`
      （ASCII 表、CJK 字母/数字、表外标点/emoji 仍拒、别名不变）。
- [x] 3.5 fixture 人口与语料指纹：`crates/jarde-reader/src/classfile.rs` 的普查 pin 移到
      `(691, 2943, 282, 1827, 8)`（+4 类 / +11 Code 体，逐字记录在 pin 上方的注释里）；
      `tests/fixtures/corpus-fingerprint.json` 再生（纯新增 10 个文件项；README 按 `md` 排除规则不入表）。

## 4. 验证与门禁

- [x] 4.1 双 javac 腿锚：声明层呈现 `变量`/`描述`/`方法`/`内部类`/`名字`（无 `__`、无
      “is not a Java identifier” 标记）；整类剥离注释后 `javac --release 8 -encoding UTF-8` exit 0；
      `java -Xverify:all` 输出 `变量=1/42/中文`，与冻结 class 自身输出逐字节相同。
- [x] 4.2 可证伪：把 `names.rs` 单独 `git stash` 到修前状态后，本片 4 个新测试中 3 个失败
      （锚、往返、局部名对照），ASCII 对照测试照旧通过——新增检查确实捕捉该回归。
- [x] 4.3 零回退扫描：2721 类 before/after 逐字节 diff，语料中 **只有 2 个** 类的文本变化（`UT`、`UT$内部类`，
      即本片锚），其余 2679 份（含全部 ASCII 语料）逐字节未动；非零退出集合两侧相同（40/40）。
- [x] 4.4 门禁：`cargo test --workspace --tests --locked --no-fail-fast`、`cargo fmt --all -- --check`、
      CI 逐字 clippy（`.github/workflows/ci.yml` 46–76）、`openspec validate --all --strict`、
      `git diff --check`、语料指纹再生后 verify —— 结果见 `verification.md` 门禁节。
- [ ] 4.5 root 独立复核（锚复现、语料扫描抽样、合并态门禁）。**留 root**
