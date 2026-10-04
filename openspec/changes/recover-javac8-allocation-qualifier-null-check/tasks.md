## 0. 前置确认（root 已完成）

- [x] 0.1 root 已实测钉死第一道门为 `new@1` 实例读者门（`init.rs:687`，诊断逐字匹配 `694` 模板、readers=`[23]`=真 javac 8 的 `dup`），并核实 `renders_its_reads`（1331-1347）不接受 `Operation::Duplicate`。取证见 [getClass 片 tasks 3.5](../recover-javac8-getclass-null-check-idiom/tasks.md) 与 [bridge-superclass-rawheader-misdispatch](../../evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/README.md)。（root 已完成）
- [x] 0.2 root 已核实可复用的谓词 `facts.rs::is_discarded_null_check` + `NullCheckSpelling{RequireNonNull,GetClass}`（getClass 片交付、已合入主线），以及既有先例"构造器自有 `dup` 不是读者"（`init.rs:676-678`）。（root 已完成）
- [x] 0.3 **未决**：第一道门究竟是读者门 687 还是 `verify_member` 分配限定符臂的 `single_use_at`（`init.rs:942`，javac 8 的 `dup(23)` 使外层实例有 2 个 use 而它要求恰 1 个）——root 只测了表层诊断码、未插桩确认内部先后。**这是 task 1.1 的首要义务**（见 design 决策 1）。（留实现者插桩）

## 1. 取证与基线

- [ ] 1.1 **插桩确认第一道门（design 决策 1 的前提，不得跳过）**：以 CF11DBG 式临时插桩（或增量实验）确认真 javac 8 的 `N1.main`（`new N1().new Inner(3)`）究竟先撞 `verify_member`（506，`single_use_at` 失败）还是读者门（681-687）。把插桩转录存入证据目录。**据结论选择 design 决策 1 的方向 A / B / 两者**，并在报告中说明为何该方向是第一道门的正解。root 倾向方向 A（三元组归站点自有，不碰 `single_use_at`），但**以插桩为准**。
- [ ] 1.2 复现 getClass 片的主锚实验：以 Corretto 1.8.0_432 编译冻结 `N1.java`，记录修复前 `N1.main` 的源码区 quotes（应为 13）、`new N1().new Inner(3)` 未折叠、拒绝码 `jre_new_shape`（诊断含 "read only by instructions this build quotes (BCIs 23)"）。
- [ ] 1.3 冻结 fixture（**真 javac 8 编译**，目录内记录编译器版本与命令行）：
  - **正例 A（分配限定符 + int 实参）**：`N1` 族（`new N1().new Inner(3)`，root 已核实其 BCI 23/24/27 为 `dup/getClass/pop`）。行为基线 `10`/`7`/`13`。
  - **正例 B（分配限定符 + 无实参）**：`new Outer().new Inner()`（无 int 实参，Open Question 3 的对照——确认带实参与无实参形都被修）。标识符与形态须与 A 不同（handoff "验收锚不得是唯一正例"）。
  - **负例 C（用户显式 getClass）**：`o.getClass(); return o.new In();`（getClass 片已冻 `G.java`，可复用或另冻）——用户语句保留、不误折叠。
  - **负例 D（外层实例多处真实消费）**：外层实例被非 null-check 的多处读取（如 `Outer o = new Outer(); o.f(); o.new In();`）——`single_use_at`/读者门核心不变量不破，仍拒。
  - **负例 E（null-check 返回值未被丢弃）**：若可构造 `Outer x = Objects.requireNonNull(o)` 保留返回值的形，其 `dup`/读者不归站点、仍拒（若 javac 产物不可表达则用合成探针 + javap 存证，说明为何不可表达）。
- [ ] 1.4 记录修复前基线：`tests/fixtures` 内分配限定符 `getClass` 形的类数（root 普查 DT-03 时 `getClass` 形为 0 类，本片新增真 javac 8 fixture 后应 >0）。

## 2. 判据扩展

- [ ] 2.1 按 1.1 插桩结论实现方向 A（读者门：把被丢弃的 null-check 三元组归消费站点自有）或方向 B（`verify_member` 分配限定符臂：放宽 `single_use_at` 接受被丢弃的 null-check `dup`）或两者。**无论哪个方向，识别 null-check 一律调用 `facts.rs::is_discarded_null_check`（决策 2），不得新写 `getClass`/`requireNonNull` 字面比较。**
- [ ] 2.2 **核心不变量不得破**：外层实例被**多处真实消费**（非被丢弃的 null-check）时仍拒——`single_use_at`（942）与读者门 `readers.len() != 1`（703）的"实例只有一个 Java 拼写位"语义必须保住。放宽只针对"被 `is_discarded_null_check` 证明、且返回值被 `pop` 丢弃"的三元组。
- [ ] 2.3 **不引入 `java_release`/`major_version` 判据**（决策 3）。实现中若出现按版本分支即范围外，须删除。
- [ ] 2.4 参数限定符形（getClass 片已修）与隐式 this 形**逐字零回退**：本片只动分配限定符路径，`N1x`/`Wrap` 族渲染不变。
- [ ] 2.5 拒绝文本与注释随判据同步（若改了读者门/`verify_member` 的拒绝消息，须使其不指称单一拼写，同 getClass 片决策 4）。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（基线 **301 目标 / 2970 passed / 0 failed**；已知 flake 家族见 handoff.md——含 `p4_plugins` 计时、`bulk_recovery_delivery.rs:421`、`p3_two_exit_return`/`backward_second_entry` scratch `AlreadyExists`、`ordinary_generic_projection` 等，**单测复跑两轮判定**）；fmt；clippy 从 `ci.yml` 46–76 行逐字生成（含 `--all-features`、29 项 `-A`、`-D warnings`）；`openspec validate --all --strict`（**274 项**）；`git diff --check`（证据里 `diff` normal-format artifact 的 `< ` 空行是忠实记录**不得删**）；**新增 fixture 后再生 corpus fingerprint** 并确认 `corpus_files_match_the_recorded_fingerprint` 通过。
- [ ] 3.2 三方对照（原 class / 固定 JADX dev / Jarde 重编，均 `java -Xverify:all`）：正例 A/B 渲染源集 `javac --release 8` **exit 0**、运行输出与原 class 逐行一致（A 为 `10`/`7`/`13`）；**双腿**（真 javac 8 与 javac 23 `--release 8`）都通过。
- [ ] 3.3 javac 9+ 零回退：`N1.java` 的 javac 23 腿渲染逐字节不变；getClass 片 8 个 `requireNonNull` fixture + `N1x`/`Wrap` 参数限定符形逐字节不变；负例 C/D/E 各按预期（C 不误折叠、D/E 仍拒）。
- [ ] 3.4 corpus 双腿扫描：差异类只应是此前因分配限定符 `getClass` 形被拒的类；出现任何 `requireNonNull` 形或参数限定符形差异即回退失败，**停下报告**。
- [ ] 3.5 root 独立复核：第一道门的插桩转录、方向选择的正确性、`is_discarded_null_check` 复用（无第二套拼写判据）、核心不变量（多处真实消费仍拒）未破、无版本判据、双腿行为、负例 C/D/E；更新 **DT-03** 账本（`declarations-types.md` 从"部分修复"改为"已修复"）与 `summary.md` 的 DT-03 状态归属（"有差距"→"冻结差距已修复"，计数 45→46、1→0）。（留 root）
