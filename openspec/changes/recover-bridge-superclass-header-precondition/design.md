## Context

[实证](../../evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/README.md)：`Spec extends Outer.Box<String>` 被 jarde 渲染为裸头 `extends Outer$Box` + 两桥隐藏，重编后 javac 只再生协变 `get` 桥、不再生参数收窄 `set` 桥 → 经 `Outer.Box` 擦除引用 `set("x")` 路由到 `BOX.set(Object)` 而非 `SPEC.set(String)`（静默错值）。对照 `Specialized extends Box<String>`（顶层父名无 `$`）渲染为参数化头、两桥都再生、擦除派发正确。

### 精确落点（root 已定位，含已验收 bridge 片的现有守卫；**行号为 ncl 合并后主线 `78db127b` 的当前值，会随代码漂移——以锚点名为准，行号仅作导航**）

- **桥准入前置守卫**：`src/facade.rs` 中 `if let Some((owner, use_kind)) = &inherited_owner && *use_kind == ReferenceUse::InvokeInterface && parameter_cast_form { … bridge_interface_contract_generic … if owner_generic { refuse; continue } }`（`grep -n "use_kind == ReferenceUse::InvokeInterface"` 的第二处命中，当前约 29909 行）。父类边（`InvokeVirtual`，见 `direct_supers.push((super_class.raw().clone(), ReferenceUse::InvokeVirtual))`，当前约 29730 行）不进此块，故参数收窄桥在裸父头下仍被隐藏。
- **接口边判据的既有实现**：`fn bridge_interface_contract_generic`（当前约 30063 行）——读类 `Signature`、判该接口是否被拼为带类型实参。本片需要一个**父类边对称件** `bridge_superclass_contract_generic`：读同一类 `Signature` 的 `superclass` 段、判其是否带类型实参。父类边**无需**解析父类定义（类自身 `Signature` 的 superclass 段已陈述 `LOuter$Box<Ljava/lang/String;>;` 或裸 `LOuter$Box;`）。
- **`parameter_cast_form` 已在准入段计算**：`let parameter_cast_form = bridge_parameters != target_parameters;`（当前约 29578 行）——即桥形是否参数收窄。协变返回桥 `parameter_cast_form == false`，天然不进新守卫（保持隐藏、行为正确）。
- **父类头退裸的根因落点**：`src/class_source.rs` 的 `project_generic_signature` 内 `if parent.binary_name.contains(&b'$') || !prove_direct_parent(...)`（当前约 6444 行）——含 `$` 的父类名被拒，故嵌套泛型父类退裸头；总门 `if parsed.type_parameters.is_empty() && !parameterized_superclass { return Ok(None) }`（当前约 6372 行）不看 interfaces。

**第一个取证义务**：(a) 确认 `bridge_interface_contract_generic` 的实现结构能否直接抽出父类对称件（两者都读类 `Signature`，只是取 interfaces 段 vs superclass 段），还是需要新解析；(b) 确认裸父头 + 协变返回桥（`Spec.get`）确实不进 `parameter_cast_form` 分支（即新守卫不误伤协变形）——用 `Specialized`（顶层，参数化头，两桥都隐藏且正确）与 `Spec`（嵌套，裸头）双向钉死；(c) 确认 `BR$StrBox` 从"桥隐藏"变为"桥可见"后仍 `javac` 通过（参数 cast 桥声明是合法重载，见 bridge-method-patrol README 结论），行为不变（其 set/get 空体）。

## Goals / Non-Goals

**Goals:** 参数收窄桥在裸父类头下保持可见（响亮、可编译、行为正确）；协变返回桥隐藏不受影响；接口边既有前置零回退。**Non-Goals:** 父类参数化投影覆盖嵌套父名（`class_source.rs:6432` 的 `$` 拒绝——那是姊妹片 `recover-parameterized-superclass-nested-headers` 的根治域，本片不碰）；接口边判据改动（已验收）；门 1/门 2/`negative/`/`orphan/` 负例改动。

## Decisions

1. **前置扩到父类边，与接口边对称**：新守卫条件 `use_kind == InvokeVirtual && parameter_cast_form && superclass_contract_is_generic_but_header_raw` → 拒绝隐藏、保持桥可见。判据源是**类自身 `Signature` 的 superclass 段**：若 Signature 陈述父类带类型实参（`LOuter$Box<Ljava/lang/String;>;`）而投影后类头是裸形（`extends Outer$Box`，即父类参数化投影因 `$` 拒绝而回退），则参数收窄桥不可隐藏。若类无 `Signature` 或 superclass 段无类型实参（本就非泛型父类），不触发（该形隐藏与否与本片无关，按既有）。
2. **协变返回桥不受影响**：`parameter_cast_form == false` 时（协变返回，`String get()` vs `Object get()`）不进新守卫——裸头下它仍是同擦除签名覆写、javac 仍再生桥，隐藏安全（`Spec.get`/`Specialized.get` 实测 `SPEC.get ran` 两侧一致）。这是与接口边前置的关键差异：接口边前置也只对 `parameter_cast_form` 生效（29875），本片保持同一约束面。
3. **"桥可见 + 裸头"是正确终态**：拒绝隐藏后，`Spec` 呈现 `extends Outer$Box` + `set(String)` + 可见的 `set(Object)` 桥（转发 `set((String)arg)`）——`javac` 通过（参数 cast 桥是合法重载），经擦除引用 `set(Object)` 调用命中可见桥 → 转发 `set(String)` → 行为正确。这比"隐藏桥 + 裸头"（错值）和"隐藏桥 + 参数化头"（需姊妹片）都更保守但正确。
4. **验收锚定**：`Spec`（嵌套泛型父、可观察 set 体）→ 桥可见、擦除派发 `SPEC.set(String) ran`（与原类一致）；`Specialized`（顶层泛型父）→ 两桥仍隐藏、行为一致（零回退）；`BR$StrBox`（嵌套泛型父、空 set 体）→ 桥可见、`javac` 通过、行为不变（`s`）；协变返回正例（`BR$Base.next`）→ 隐藏不变。

## Risks / Trade-offs

- **corpus 面变化**：`BR$StrBox` 等父类体不可观察的形从"桥隐藏"变"桥可见"，corpus 双腿扫描会出现差异（桥声明从无到有）——须如实记录，且确认差异**仅**桥成员声明、无行为变化（其空体两侧一致）。这是本片预期的差异，不是回归。
- **前置过宽误伤协变返回桥** → 决策 2 的 `parameter_cast_form` 约束钉死；`Specialized.get`/`Spec.get` 双向正例证明协变形仍隐藏。
- **与 `recover-parameterized-superclass-nested-headers`（姊妹片）的协调**：本片堵行为洞（拒绝隐藏），姊妹片做根治（父类头参数化投影覆盖嵌套名，使桥可安全隐藏）。两片串行：本片先落地（消除已合入代码的静默错值），姊妹片随后（让该形恢复"隐藏桥 + 参数化头"的更优呈现）。本片的前置在姊妹片落地后自动失效（父类头带类型实参 → `superclass_contract_is_generic_but_header_raw` 为假 → 允许隐藏），无需回退本片。
- **打破一个已验收 bridge 测试是预期效果（root 已核实，勿误判为回归）**：`tests/class_source.rs:7439-7445`（`recover-bridge-admission-gates` 的 BR$StrBox 测试）断言两个桥都 `admitted`/`projected` 且文本不含 `void set(java.lang.Object`。但 root 用 javap 核实 `BR$Box<T>` 是泛型父（Signature `<T:Ljava/lang/Object;>…`）、源码 `StrBox extends Box<String>`、binary 名 `BR$Box` 含 `$` → 父类投影退裸头 → BR$StrBox 的 `set(Object)` 参数收窄桥**恰命中本片新前置** → 从隐藏变可见。故那三条断言（当初断言的正是 bug 行为）必须**更新为修复后期望**：`get()` 协变桥仍隐藏、`set(Object)` 桥现在可见（`!admitted`/`!projected`、文本含 `void set(java.lang.Object`）、源级 `set(String)`/`get()` 仍在。**不得为让旧断言变绿而削弱父类边前置**（那等于留着 bug）。BR$StrBox 行为不变（`BR$Box.set` 是空体），故该形须用**可观察** set 体（证据目录 `Spec`，`Outer.Box.set` 打印）实测 `SPEC.set(String) ran` 才算证明修复。
- **接口边前置的对称性**：本片把父类边纳入后，接口边与父类边判据应结构对称（都读类 Signature、都只对 parameter_cast_form 生效）；实现时抽公共 helper 避免两套逻辑漂移，但不得改动接口边已验收的拒绝文本。
