# `-args` 恢复记录：自定义参数枚举构造的常量折叠（2026-10-01）

实现 `recover-proved-enum-arbitrary-arguments`（基线 `1952cbfe`）。巡查结论（见[上级 README](../README.md)）定位的 descriptor 白名单根因按 tasks 1.1–3.2 闭合；3.3 root 复核未做。

## 1. 拒绝点定位（任务 1.1）

- fixture 复核：`fixture/N0/N1/N2/N3.class` SHA 与 `results/fixture-sha256.txt` 逐字一致（复放命令 `shasum -a 256`）。
- 拒绝点：`src/enum_constants.rs` 单构造分支，`refuse("the enum has an unsupported constructor descriptor")`（基线行号约 1668–1674，四常量 `CTOR_DESCRIPTOR`/`DELEGATING_CTOR_DESCRIPTOR`/`STRING_CTOR_DESCRIPTOR`/`STRING_VARARGS_CTOR_DESCRIPTOR` 于文件头 32–36 行）。
- 改动前二进制（`1952cbfe` 独立 worktree 构建）逐类复放输出与 `results/*.jarde.java`、`fixture/N0.jarde.java` 逐字一致（零漂移确认）。
- ctor 体/常量步骤证明的参数化点：单构造 `prove_constructor`/`prove_string_constructor`/`prove_string_varargs_constructor`（各固定形专属）与 `prove_initializer_prefix` 的 `EnumSourceArgument` 分派；委托链 `prove_terminal_constructor_body` 不变。

## 2. grammar 参数化落点与四固定形关系

**并存，不统一。** 新增 `parse_arbitrary_ctor_descriptor`：descriptor = `(Ljava/lang/String;I` 前缀 + 1..=3 个用户参 + `)V`，每参 ∈ {`B`,`C`,`S`,`I`,`Z`, `Ljava/lang/String;`, 任意 `L…;`}（`J`/`F`/`D`/数组显式拒绝）。四固定形 descriptor 在白名单处先匹配原路径（flags/Signature 检查/`expected_ctor_field`/`constructor_calls`/字面量计费/`projected_constructor_descriptors` 六处分支均保持原顺序，新路径仅以 `else if` 追加），理由：

1. 固定形各自携带更宽的既有证书契约（`(String,int)` 的任意 int 表达式、String 三元、varargs 数组、精确序数委托前缀），统一进新 grammar 要么削弱要么重实现这些行为，"N0 与四固定形逐字不变"以原路径保留最直接达成；
2. 新 grammar 按参数 descriptor 分类实参期望（int 族→字面量、String→ldc、对象→getstatic|null），实参位与 ctor 体第 i 用户参逐条对齐（`prove_arbitrary_user_constructor`）。

注意：`(Ljava/lang/String;ILjava/lang/String;)V` 同时可被新 grammar 解析（单 String 参），但白名单先命中固定形，行为不变（单测 `arbitrary_ctor_descriptor_grammar_parses_only_the_admitted_tails` 钉死两种解析的存在性与优先级）。

## 3. 折叠与呈现（任务 2.1/2.2）

- 命中输出（`recovered/`，SHA 见 `recovered-sha256.txt`）：
  - `N3$Simple`：`A((byte) 1, "x"),\n    B((byte) 2, "y");`，ctor 呈现 `private N3$Simple(byte arg0, java.lang.String arg1) { this.num = arg0; this.s = arg1; }`；
  - `N3$Refs`：`A(N3$Simple.A),\n    B(N3$Simple.B);`；
  - `N1$Numbers`：`ONE((byte) 1, N1$Numbers$NumString.ONE),\n    TWO((byte) 2, N1$Numbers$NumString.TWO);`；嵌套 `N1$Numbers$NumString` 输出与改动前逐字一致（N0 型）。
  - values/valueOf/clinit/$values/$VALUES 归入 enum 呈现（文本无 `valueOf(`、无 `$VALUES`）。
- 窄化拼写按参数 descriptor：`B`→`(byte) v`、`S`→`(short) v`、`I`→裸值、`Z`→`true`/`false`（非 0/1 拒绝）、`C`→char 字面量（可打印 ASCII 与引号/反斜杠转义；其余值含负数与控制字符用 `(char) v`——javac 常量折叠等价形）。getstatic 拼 `Owner.name`，owner 走既有 descriptor→source 拼写通道（`type_of_component().spell()` 取末段，与 enum-int-arguments `proved_int_static_field` 同款），不跨类读取；null 拼 `null`。提案示例 `A(Simple.A)`/`NumString.ONE` 的源级简单名形态在既有逐类拼写通道下落为 `N3$Simple.A`/`N1$Numbers$NumString.ONE`（与 field/enumswitch 呈现同款，保持 `$` 全名）。
- ldc 实参注：新 grammar 的 int 族字面量含 `ldc int`（proposal 明列）；固定 `(String,int)` 路径维持其原 int 表达式契约不变（`proved_int_literal` 未动，新 `proved_int_family_literal` 只服务新路径）。

## 4. 负例/变体（任务 1.2，verifier 有效，`java -Xverify:all` 通过）

源码 `variants/ArgsRefusals.java`；前后文本 `variants/{before,after}/`。改动前全部 `refuse("the enum has an unsupported constructor descriptor")`；改动后保持逐字段呈现，拒绝原因（crate 内证明文本）：

| 形态 | 改动后拒绝原因 |
| --- | --- |
| 实参种类不符（Object 参收 ldc String） | `constant 0 argument 0 does not match its constructor parameter` |
| 实参种类不符（String 参收 getstatic） | `constant 0 has no closed scalar String argument` |
| ctor 体额外语句（println 前置） | `the arbitrary constructor Code is incomplete, noncanonical, or has handlers` |
| >3 用户参 | `the enum has an unsupported constructor descriptor`（grammar 上限拒绝） |
| `long` 参（J 形态，未做登记） | `the enum has an unsupported constructor descriptor` |

char/boolean/short/int/null/getstatic 正变体在 `variants/ArgsMatrix.java`（折叠拼写 `R1(BytesText.A)`、`R3(null)`、`THREE((char) 65535)`、`NO(false)`、`BIG((short) 300)`、`MID(5)`），重编运行一致。

## 5. 三方对照（任务 3.2）

`runs/`（SHA 同文件）：原 class / 固定 JADX（1.5.x，`--no-res`，Java 8 重编）/ Jarde 折叠重编，`java -Xverify:all`：

- `n3`：三路输出逐字一致（`ad0fad…`）。
- `n1`：三路输出逐字一致（`e18650…`）。
- `args-refusals`：原 class 与 JADX 一致（`caaffc…`）；Jarde 按设计保持逐字段（"not claimed to compile"），无重编宣称。
- `args-matrix`：原 class 与 Jarde 折叠重编逐字一致（`5fbced…`）；**JADX 输出无法重编**（`ShortArgs.java` 裸 `-300` 丢窄化，`runs/args-matrix.jadx-compile-failure.log`），与本片要修复的形态相同，JADX 按巡查约定为参照非语义正例。

## 6. 门禁（任务 3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：exit 0，272 个测试二进制 **2756 passed / 0 failed**（基线 2748 + 本片新增 8）；fmt 重排后对最终状态复跑仍 2756/0。
- `cargo fmt --all -- --check`：PASS。
- clippy：`.github/workflows/ci.yml` 完整 30 项 `-A` 清单 + `-D warnings`（`cargo clippy --workspace --all-targets --all-features --locked`）：exit 0。
- `openspec validate --all --strict`：**232 passed, 0 failed**。
- 磁盘纪律：构建/测试前 `df -h /` 检查；低于 15Gi 时 `cargo clean`（基线 worktree 构建后即清，主 target 在 workspace 测试后清理重建）。

## 7. 边界与遗留（如实登记）

- `double`/`float`/`long` 字面量实参、数组参数不在 grammar（出现即保持逐字段并在上表登记）；匿名子类常量（N2$Operation）不在本片，N2 输出与改动前逐字一致；`>3` 用户参按需另扩。
- int 族参数只收字面量（design 决策 1）：getstatic int 字段传给 int 参仍拒绝（固定 `(String,int)` 路径的既有能力不因此变化）。
- 构造器参数注解（`RuntimeVisibleParameterAnnotations` 等）非空时新路径放弃折叠（声明合成会丢失注解）；ctor 上唯一可吸收 marker 为 `jvm_signature_erasure_mismatch`。
- 事实性边界：嵌套 enum family 装配（`recover-proved-nested-enum-source`）对新 grammar 组沿用同一投影，其固定样例不含任意实参形态，行为不受影响；新增可折叠输入只会来自此前整体拒绝的类。
