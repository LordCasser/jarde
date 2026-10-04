# results-gates/ — `recover-bridge-superclass-header-precondition` 验收证据（coder，2026-10-04）

基线 `ca135c9f`（含已验收 `recover-bridge-admission-gates` 桥片）；修复后为同一 worktree 加父类边前置的实现。双腿二进制：基线二进制由 `git stash` 分离 facade.rs 改动后重建（SHA256 `35d7c158fb94169867af88fbee92aa8b6061cf9360d2bfc1b9fe8e96dc9350e6`），修复二进制由最终 facade.rs 构建（SHA256 `631ea00ca915a6e9d5904a1f52766ba768a4e59717a7823d843d4c7786f2d561`）。基线渲染经与 root 存档 `results/rendered-Spec.java` 对照核实（代码行逐字一致，仅注释行存档差异）。全部命令在本机 JDK（javac/java `--release 8`）实测。

## 一、四向呈现与行为对照

### 1. 正例 `Spec`（嵌套泛型父 `Outer$Box` + 可观察 set 体）——修复目标

- 修复前渲染（`rendered-Spec-gates-baseline.txt`）：裸头 `extends Outer$Box`（Signature `LOuter$Box<Ljava/lang/String;>;` 因父名 `$` 拒绝投影，拒绝文本与 root 存档逐字同），`set(Object)` 参数收窄桥**隐藏**（projected-bridge 注释标记）。
- 修复后渲染（`rendered-Spec-gates-fixed.txt`）：类头不变（本片不改投影），`set(Object)` 桥**可见**（`void set(java.lang.Object arg1) { this.set((java.lang.String) arg1); }`），协变 `get` 桥仍隐藏。渲染 diff **仅** set 桥一处；`get` 桥的 projected 标记行逐字不变（协变零回退）。
- 三方运行（家族 jar：原 `Outer`/`Outer$Box`/`Drv` + 重编 `Spec`）：

| 腿 | `b.set("x")`（经 `Outer$Box` 擦除引用） | `b.get()` | javac | 重编 ACC_BRIDGE |
| --- | --- | --- | --- | --- |
| 原类（`run-nested-original-gates.out`） | `SPEC.set(String) ran` | `SPEC.get ran` | — | — |
| 基线渲染重编（`run-nested-baseline-misdispatch-gates.out`） | **`BOX.set(Object) ran`（错值复现）** | `SPEC.get ran` | exit 0 | 1（仅协变 get） |
| 修复渲染重编（`run-nested-fixed-dispatch-gates.out`） | **`SPEC.set(String) ran`（与原类一致）** | `SPEC.get ran` | exit 0 | 1（javac 再生协变 get 桥；`set(Object)` 已是显式普通成员） |

- 重编成员表见 `recompiled-Spec-gates-{baseline,fixed}.javap.txt`（verbose 版同名 `-verbose` 文件）：基线重编 `Spec` 无 `set(Object)`（裸头下 `set(String)` 是重载 → javac 不再生参数桥）；修复重编含显式 `set(Object)` 转发体。

### 2. 顶层对照 `Specialized`（父名无 `$`，零回退）

- `rendered-Specialized-gates-{baseline,fixed}.txt` **逐字节相同**（`cmp` 通过）：参数化头 `extends Box<java.lang.String>`、两桥均隐藏。
- 重编（`recompiled-Specialized-gates-fixed.javap.txt`）：javac 从参数化头再生两桥（ACC_BRIDGE=2）；运行 `run-toplevel-original-gates.out` / `run-toplevel-fixed-recompiled-gates.out` 均为 `SPEC.set(String) ran` / `SPEC.get ran`。

### 3. 协变返回对照（不受前置影响）

- `Spec.get`：见上表，两腿 `SPEC.get ran` 一致；渲染中 get 桥在基线与修复均隐藏。
- `BR$Base.next` / `BR$Impl`：接口边与协变返回的既有判据零改动——`br_family_bridges_admit_through_the_extended_gates`、`br_family_negative_shapes_keep_their_refusals`、`the_multilevel_covariant_bridge_walks_two_interface_edges`、`the_parameter_cast_admission_walks_the_snapshot_chain_and_respects_its_edges` 全部按既有断言通过（接口边拒绝文本逐字未动）。

### 4. `BR$StrBox`（已验收片正例，预期呈现变化：桥从隐藏变可见）

- 修复后渲染（`rendered-BRStrBox-gates-fixed.txt`）：裸头 `extends BR$Box` 不变 + `set(Object)` 桥可见（转发体）+ `get` 桥仍隐藏。`BR$Box` Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;`（javap 实证，泛型父）、binary 名含 `$` → 恰命中新前置。
- 三方运行（`run-br-family-original-gates.out` / `run-br-family-fixed-recovered-gates.out`）：`BR$Base` / `s` 两侧一致，`javac --release 8` exit 0（参数 cast 桥是合法重载，见 bridge-method-patrol README 的 root 事实修正）。**仅桥成员声明从无到有，无行为变化**（`BR$Box.set` 空体，两侧 set 体同为空）。
- 单元断言同步：`tests/class_source.rs` 的 BR$StrBox 断言由"两桥全 admitted/projected"更新为按桥分断（get 隐藏 / set 可见 + 新拒绝文本）；`the_parameter_cast_admission_walks_the_snapshot_chain_and_respects_its_edges` 的 set_proof 断言同步为"拒绝文本是新前置而非遍历回边"。这是**预期效果**（旧断言断言的正是 bug 行为），不是对修复的削弱。

### 固定 JADX 对照（仅对照，不作准入）

`jadx-Spec-gates-reference.txt` / `jadx-BRStrBox-gates-reference.txt`：JADX 走"嵌套名参数化头（`extends Outer.Box<java.lang.String>`）+ 丢桥"路线——其重编依赖 JADX 自己的嵌套名拼写体系。jarde 的 binary 名拼写下，本片的"可见桥 + 裸头"是该约束面前的保守正确终态；参数化头路线归姊妹片 `recover-parameterized-superclass-nested-headers`。

## 二、corpus 双腿扫描

方法：corpus = `tests/fixtures` + `fuzz/corpus`（排除 `out/`/`target/`）。散类（445 个可读类）先按 `list-classes` 读出的内部名重命名staging、按目录打包家族 jar（家族上下文才触发同类桥准入；条目名与内部名一致保证按名可寻址）；归档（6 个）按存储原样渲染。每类双腿（基线/修复二进制）`class-source --format text` 渲染并 `cmp`。

- 结果（`corpus-scan-summary.tsv`，完整文本 diff 在 `corpus-scan-diffs.txt`）：**448 次渲染尝试，441 渲染成功，439 逐字一致，2 处差异，7 处双腿皆不可按名寻址**（`Holder`/`Owner`/`Other`/`AnonymousSuperDispatch`/`matrix/UsePlainRaw`/multi-release 两版本条目——基线与修复二进制同样失败、失败原因逐字节相同，属既有绑定形状，与本片无关）。
- **2 处差异均为"父类边参数收窄桥从隐藏变可见"的形**：`BR$StrBox`（br-family）与 `Spec`（本片新 fixture）。diff 逐行核对：**唯一**变化是 `set(Object)` 桥成员声明从 projected 注释标记变为可见转发体（`this.set((java.lang.String) arg1);`），无其它行变化。行为无变化由第一节四向运行对照证明（BR$StrBox 空体两侧一致；Spec 观察体修复后派发正确）。

## 三、门禁实录（真实数字）

- `cargo test --workspace --tests --locked --no-fail-fast`：全绿（最终数字见交付报告；`class_source` 93/93 含新增 `the_superclass_header_precondition_keeps_a_raw_header_parameter_bridge_visible`；该新测试在仅回退 facade.rs 时于 `!set_proof.admitted` 失败——回归检查可在旧行为上失败）。
- fixture 群体计数：新增 7 个冻结类使 `repository_class_fixtures_validate_without_false_target_rejections` 的 pinned 元组从 `(465, 2172, 251, 1679, 8)` 重测为 `(472, 2193, 251, 1679, 8)`（+7 类 +21 体；handler/branch/subroutine 不变）——既定协议，注释留档。
- `cargo fmt --all -- --check`：通过（先 fmt 修正了一处换行）。
- `cargo clippy --workspace --all-targets --all-features --locked --`（ci.yml 46–76 行逐字 29 项 `-A -D warnings`）：exit 0。
- `openspec validate --all --strict`：269 passed, 0 failed。
- corpus 指纹清单 `tests/fixtures/corpus-fingerprint.json` 再生：仅新增 7 个冻结类条目。

## 四、判据与实现要点（供 root 复核 3.3）

- 判据源两层：**(a)** 类自身 `Signature` 的 superclass 段带类型实参（`bridge_superclass_contract_generic`，与 `bridge_interface_contract_generic` 对称；段测试与投影的 `parameterized_superclass` 同形防漂移）；**(b)** 投影后类头未带该类型实参——判据 (b) 不在准入处重算投影逻辑，而是管线直接传类头投影的既成结果 `class_scope.is_some()`（`project_generic_signature` 仅在**成功提交参数化类头**时置位；非预算 Err 降级为 `Ok(None)`；预算停止 fail-closed）。故：`Specialized`（投影成功）短路放行零回退；`Spec`/`BR$StrBox`（`$` 拒绝退裸）触发拒绝；姊妹片落地后父类头带实参 → `class_scope` 置位 → 前置自动失效，无需回退本片。
- 协变返回桥 `parameter_cast_form == false` 天然不进新守卫（与接口边同一约束面）。
- 非泛型父类：Signature 无/段无实参 → 判据 (a) 为假 → 不触发（按既有）。javac 本就不为非泛型父类的参数重载发桥，该形在真实字节码中不出现参数收窄桥。
