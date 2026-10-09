# 基线证据复用审计 v1

结论：`verified_historical_identity_with_limits`。这里只重算并核对已有证据；没有运行Java、JADX、Cargo或Git，也没有产生本片fresh三方结果。

## 输入与闭集

当前冻结direct fixture：`/Users/lordcasser/workspace/projects/jarde/tests/fixtures/p3-heterogeneous-array-initializers-v3/direct`。分母为 `Base、DerivedA、DerivedB、LocalInterface、Main、Mid` 六类 × Corretto 8 / OpenJDK 23 两腿，共12个输入class。numeric历史jar、旧EM-18 Jarde输入jar、JADX四个direct profile中的class hashes均与当前冻结classes相符。逐类哈希见JSON。

- numeric旧结果：`/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-constructor-primitive-conversion-arguments/results/legacy-regressions-root-v1/manifest.json`，SHA-256 `dc8a09f2b9d34bd50f2e8b43f5fbcfda23e2f7bfdca58c9970cf81be962528eb`，304条记录闭合=True。其冻结引用的两个历史manifest也重新核对了SHA及各自目录闭集。旧CLI `/private/tmp/jarde-constructor-primitive-conversions-cli-v1` SHA-256 `3e4b615e0131d041ecc47c72c0131a3bbd0f5fc9042d8f91b2aa21f26baf6a95` 与manifest相符。
- 旧EM-18 Jarde：`/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-heterogeneous-array-init/results/candidate-v1-fixture-v3/manifest.json` SHA-256 `d3dd331861d79d32cc64b2fd2e2c3677606868d54767e157fa99e5db7f508b87`，110条闭集=True；replay-v2 256条闭集=True。两腿生成了6类源，但编译exit=1、没有candidate runtime。
- 旧JADX归档：`/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-heterogeneous-array-init/evidence/jadx-fixture-v3-reference.tar.gz` SHA-256 `7d828c23041fddbb9b895bd45c0683e6b3c06c17dde016ea6f3da24fcb6124fc`，78261字节；213普通文件与211 payload条目+2个manifest闭合=True。root核验SHA `353a225a9894a6af2e812f8b1928957f70a87948160eb4e50ed63e4ecea6cf9c`，issues=[]。

当前direct六个Java源hash也与旧JADX保存的original-source fingerprints匹配；原始输入与六类分母身份可历史复用。 JADX归档没有携带原程序直接运行的raw文件，但原始命令双流在当前冻结fixture的pathfix logs中存在，逐SHA与JADX原始记录/命令manifest相符；具体路径和bytes见JSON。

## JDK、命令与双流

- `javac8`：javac 1.8.0_432；openjdk version "1.8.0_432"
OpenJDK Runtime Environment Corretto-8.432.06.1 (build 1.8.0_432-b06)
OpenJDK 64-Bit Server VM Corretto-8.432.06.1 (build 25.432-b06, mixed mode)。冻结pathfix证据记录了JDK tool hashes、空classpath/sourcepath javac、`java -Xverify:all -cp ... Main`和`javap -c -p Main`。原始Main exit=0；stdout SHA `fd3a20051507b79c6537ccc6fcc24a40fb1bb10e7fda4ea01ae06a08fdc0a71b`，stderr SHA `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`，原始路径与每条命令双流hash均已重算并列在JSON。
- `javac23`：javac 23.0.1；openjdk version "23.0.1" 2024-10-15
OpenJDK Runtime Environment (build 23.0.1+11-39)
OpenJDK 64-Bit Server VM (build 23.0.1+11-39, mixed mode, sharing)。冻结pathfix证据记录了JDK tool hashes、空classpath/sourcepath javac、`java -Xverify:all -cp ... Main`和`javap -c -p Main`。原始Main exit=0；stdout SHA `fd3a20051507b79c6537ccc6fcc24a40fb1bb10e7fda4ea01ae06a08fdc0a71b`，stderr SHA `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`，原始路径与每条命令双流hash均已重算并列在JSON。

numeric旧Jarde通过固定CLI逐类运行`class-source --input <direct jar> --class <class> --policy plain-jar --format json --evidence all --release 8`，JSON保存两腿12条render命令argv/cwd/exit及双流。完整六类文件输出不代表可编译：三个grid body拒绝，候选compile两腿均exit=1，runtime为空；original raw在该numeric manifest中只有hash且`original_raw_streams_available=false`。

## 三个grid正文与BCI来源

原始源码 `/Users/lordcasser/workspace/projects/jarde/tests/fixtures/p3-heterogeneous-array-initializers-v3/direct/Main.java` SHA-256 `e4e9189086f9417c523c4db0cadce69739c66f9e6c297ccc4a7914768f29a017`。源码行号及结构在下表；BCI来自pathfix manifest `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-heterogeneous-array-init/evidence/heterogeneous-array-initializers-v3-pathfix/build-manifest-v3-pathfix.json` 的aastore记录，并通过读取保存的javap stdout按方法复核；没有运行javap。

| 方法/descriptor | 源码行与形状 | javac8 aastore BCI | javac23 aastore BCI |
|---|---|---|---|
| `numberGridDirect` `()[[Ljava/lang/Number;` | 95：Number[][] = { Integer[], Long[] } | `[19, 20, 37, 38]` | `[19, 20, 37, 38]` |
| `collectionGridDirect` `()[[Ljava/util/Collection;` | 102：Collection<?>[][] = { ArrayList<?>[], HashSet<?>[] } | `[19, 20, 36, 37]` | `[19, 20, 36, 37]` |
| `ownGridDirect` `()[[LBase;` | 117：Base[][] = { DerivedA[], DerivedB[] } | `[23, 24, 44, 45]` | `[23, 24, 44, 45]` |

parent store分别是number 20/38、collection 20/37、own 24/45。旧Jarde Main完整源保留not recovered/no statement标注；collection另含`ordinary_generic_source_unproved` Signature refusal。

## 复用边界

旧JADX direct家族每个JDK有default与rename-flags-none两档，每档完整六类源以及编译、`-Xverify:all`运行和双流。root核验接受的精确direct行为只有两个rename-flags-none JDK腿；default虽compile/runtime exit 0且有语法检查，但双流与原始不同。JADX总体为8 profile腿中4腿full-semantic accepted，72只是语法观察。

numeric Jarde和旧EM-18 Jarde输入class hashes一致，但历史Main.java生成源不同：numeric `ae93a5d17da1d9ae75016dca6f0feb9b076d02ee0d262617ae8c0918116f6d89`，旧EM-18 `97fd0d547c73372dad5594b0345e79a3eb07c83cb67f81fa2185eade5b4eedcb`；其余五类源逐字hash一致。必须按各自CLI/manifest分开引用，不能拼成同次结果。

本片没有current CLI生成的fresh Jarde/JADX/original三方结果。root后续需用前置验收后的CLI重新生成Jarde六类源、空classpath/sourcepath完整编译并-Xverify:all运行，对照原始双流与历史JADX rename-flags-none结果。旧JADX可作已核验的历史输入/source参照，须标注非fresh。
