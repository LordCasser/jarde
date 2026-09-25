# 五个 verifier-valid 错形控制（任务 1.2 的运行证据）

`build.py` 用 `javac --release 8 -g:none` 编译本目录 `fixture/` 的 Java 源，再从冻结的 `SimpleOuter`/`UseInner` class 构造五个**受控字节变体**。它逐项断言预期 opcode、Code 长度和 `InnerClasses` 表项后才修改；编译只在私有临时目录，jar 的 ZIP 时间戳固定。`SHA256.txt` 给出每个 jar 和条目 class 的 SHA-256；`toolchain.txt` 给出修前 Java/JADX/Jarde 版本，`after-jarde-cli-sha256.txt` 固定修后 CLI 二进制。`*-javap.txt` 保留完整 `javap -v -c -p`，`*-original-run.txt` 是各 jar 中 runner 的 `java -Xverify:all` 标准输出。所有七个本目录 jar（两个源码绿侧、五个错形红侧）都通过 JVM 验证，进程退出码均为 0。

| 控制 | 精确变动与 BCI | 源码绿侧 `-Xverify:all` | 错形红侧 `-Xverify:all` |
| --- | --- | --- | --- |
| 目标关系错误 | `SimpleOuter$Inner.class` 的自身 `InnerClasses` 表项把 `outer_class_info_index` 置 0；原目标 `7ce649…`，变体 `a8a1d4…`。`wrong-relation-javap.txt` 末尾仍有 `Inner` 简名，但不再有 `of SimpleOuter`。调用方 class 完全不变，构造 BCI 0/16。 | 既有 `../full-target.jar`：`7:IA`、`null:I` | `wrong-relation.jar`：`7:IA`、`null:I` |
| 外层关系错误 | 只把 `SimpleOuter.class` 的 `InnerClasses` 目标条目的 `outer_class_info_index` 置 0，保留正确的目标 `SimpleOuter$Inner.class` 与调用方 class。`wrong-outer-relation-javap.txt` 固定修改。 | 同一 `../full-target.jar`：`7:IA`、`null:I` | `wrong-outer-relation.jar`：`7:IA`、`null:I`；但以该 jar 作 classpath 重编原调用方时，`javac --release 8` 报「找不到符号：类 Inner」，见 `wrong-outer-relation-javac.txt`。 |
| 限定值与物理首参不同 SSA 身份 | `IdentityUse.make(checked, actual, value)` 的 BCI 5 从 `dup` 改为 `aload_0`；BCI 4 `aload_1` 仍是 `<init>` 的物理外层首参，BCI 6 `requireNonNull` 检查 `checked`，BCI 13 `mark`，BCI 16 `<init>`。原来源 class 与变体 class 见 `SHA256.txt`。 | `identity-source.jar`：`different:43:A`、`checked-null:43:A`、`actual-null:null:` | `wrong-identity.jar`：`different:43:A`、`checked-null:null:`、`actual-null:null:A` |
| 身份不同但保留完整检查形状 | `IdentityUse.make` 保持连续 `aload_0; dup; requireNonNull; pop`，在检查后丢弃该剩余副本，再从 `aload_1` 读取真实物理外层首参；BCI 18 才 `<init>`。此变体专门穿过形状门，检验 SSA 身份门。 | 同一 `identity-source.jar`：`actual-null:null:` | `wrong-identity-checked.jar`：`actual-null:null:A`；`checked-null:null:`。 |
| 检查晚于普通参数效果 | `LateUse.make` 保留原物理外层参数和普通参数，但在 BCI 8 执行 `mark("A")`，BCI 11/17 `swap` 保持构造器实参栈形，BCI 13 才执行 `requireNonNull`，BCI 18 `<init>`。原来源 class 与变体 class 见 `SHA256.txt`。 | `late-source.jar`：`7:A`、`null:` | `late-check.jar`：`7:A`、`null:A` |

前两个身份参数不是别名假设：runner 分别传入 bias 4、bias 40 和 null，值与 trace 将两个引用的作用分开。两种关系变体则说明 JVM 验证和执行不替静态恢复证明 Java 成员关系；目标和外层各自的 `InnerClasses` 声明都是可编译限定创建的准入事实。尤其外层变体只修改外层定义，目标自述完全正确却仍无法从外层类解析 `Inner`。晚检查变体说明即使构造器实参值相同，将源级 `outer.new Inner(mark(...))` 用于该字节码会把 null 情况的 `A` 效果移到异常之后。

JADX 1.5.6 的各 jar 反编译保存在 `jadx-*/sources/`，日志在 `*-jadx-log.txt`。修前的目标关系、旧身份、晚检查三份对照中，以冻结的原成员 class 为依赖用 `javac --release 8` 重编各 JADX 调用方均退出 0，运行逐字匹配各自错形 jar。JADX 对目标关系错误仍写出 `simpleOuter.new Inner(...)`；这份 Java 源使用**原目标**重编，不能证明错误目标关系可由 Java 源表达。旧身份变体把显式 `Objects.requireNonNull(checked)` 留在 `actual.new Inner(...)` 之前；晚检查变体先求值 `iMark`，再显式检查和构造，因此这两份运行结果保留了错形时序。JADX 输出仅作行为对照，不作为 Jarde 准入证明。

新增外层错形的 JADX 输出在 `jadx-wrong-outer-relation/sources/`。它仍写 `simpleOuter.new Inner(...)`，并同时重新生成嵌套的 `SimpleOuter.Inner` 声明；三份生成源码一起重编、运行可得原来的两行输出。但只重编调用方、以该错形 jar 为依赖时，`wrong-outer-relation-jadx-caller-only-compile.txt` 同样报类 `Inner` 无法解析。这说明 JADX 的整包重写掩盖了选定外层 class 的物理关系缺口；Jarde 的本次调用方子集必须在读取原外层定义时拒绝，而不能凭目标类自己的关系条目呈现。

更强的身份错形还暴露 JADX 的**可编译行为错误**。`jadx-wrong-identity-checked/sources/nested/IdentityUse.java` 把显式 `Objects.requireNonNull(checked)` 留在前面，却用 `actual.new Inner(mark("A", i))` 投影构造。原 class 的 `actual-null` 路径先执行 `A`，随后在构造完成后的 `inner.value()` 才因捕获的外层为 null 抛错，结果为 `actual-null:null:A`；JADX Java 8 重编后由限定创建自带的 null 检查提前抛错，结果变成 `actual-null:null:`。两份 `*-run.txt` 与编译退出码固定于本目录；不能仅以可编译判断该投影正确。

修前 Jarde CLI 对原三个错形和两个源码绿侧的 `class-source --format json` 报告在旧 `*-jarde.json`。绿侧也在 BCI 5 的 `Duplicate` 处拒绝，故这批结果只是修前基线，不能证明新增的门已生效。

启用物理目标、调用点和 AST 证明后的完整报告在 `after-jarde-*.json`。`identity-source`、`late-source` 与 `full` 绿侧均为 `structured`，各有 1 个已呈现 `new@1`；生成的 Java 8 调用方以原依赖 jar 重编退出 0，`after-jarde-*-run.txt` 与各自原 class 的 `-Xverify:all` 输出逐字一致。缺失目标、目标关系错、外层关系错均为 `fallback`、0 呈现；旧身份错形与晚检查仍保守拒绝。`wrong-identity-checked` 则保留完整检查形状后，直接命中 `jre_new_member_shape` 的「checked qualifier's two SSA copies」身份门；没有输出会把 `A` 提前消掉的限定构造。`init::tests::member_call_proof_requires_same_qualifier_and_early_check` 用这份冻结 class 作直接红/绿断言。

缺失目标控制已在 `../analysis.md`。本目录的五条字节错形均是 verifier-valid；绿侧已实际呈现，错形侧保留拒绝，目标与外层关系由 class-source 选定定义的直接测试区分，身份由完整检查形状的 SSA 负控区分。晚检查的受控字节码把效果移到检查之前，Jarde 的受限站点在检查形状门拒绝；这正是本首片规定的连续早检查准入边界，不宣称已支持任意合法检查顺序。

重建：`python3 build.py`。运行示例：`java -Xverify:all -cp wrong-identity-checked.jar nested.IdentityRunner`；`javap -v -c -p -classpath late-check.jar nested.LateUse`；`jadx -j 1 -r -d jadx-wrong-identity-checked wrong-identity-checked.jar`。修后 Jarde 对照使用 `jarde-cli class-source --input wrong-identity-checked.jar --class nested/IdentityUse --format json --evidence all --output after-jarde-wrong-identity-checked.json`。所有构造和运行命令均不执行目标代码于 Jarde 内部；JVM 命令只运行本目录受控 runner。
