## 1. 取证与基线

- [ ] 1.1 重放 [实证 fixture](../../evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/)（SHA 核对 results/fixture-sha256.txt）：用真实二进制复现 `Spec`（嵌套泛型父、可观察 set 体）的裸头 + 桥隐藏 → 家族重编 → 擦除派发 `BOX.set(Object) ran`（原类 `SPEC.set(String) ran`）；记录 `Specialized`（顶层泛型父）的参数化头 + 两桥隐藏 + 正确派发对照。
- [ ] 1.2 读 `src/facade.rs` 的接口边前置守卫块（`grep -n "use_kind == ReferenceUse::InvokeInterface"` 第二处命中，ncl 合并后约 29909 行）与 `fn bridge_interface_contract_generic`（约 30063 行），回答 design 取证义务 (a)(b)(c)：父类对称件能否抽公共 helper、协变返回桥是否天然不进 `parameter_cast_form` 分支（`let parameter_cast_form = …` 约 29578 行）、`BR$StrBox` 桥变可见后是否仍 `javac` 通过且行为不变。行号会随代码漂移，以锚点名为准。
- [ ] 1.3 冻结至少四个变体/负例：嵌套泛型父 + 参数收窄桥 + 可观察 set 体（正例，本片修复目标）、顶层泛型父 + 两桥（零回退对照）、嵌套泛型父 + 协变返回桥（不受影响）、非泛型父类 + 参数 cast 桥（若存在，按既有）；各自 `java -Xverify:all` 通过并记录实现前后呈现与擦除派发行为。

## 2. 父类边前置

- [ ] 2.1 新增 `bridge_superclass_contract_generic`（与 `bridge_interface_contract_generic` 对称，读类 `Signature` 的 superclass 段）；把接口边前置守卫块（`use_kind == ReferenceUse::InvokeInterface && parameter_cast_form`，ncl 合并后约 29909 行）扩为接口边 ∪ 父类边（design 决策 1）：`parameter_cast_form ∧ 契约属主父类边（InvokeVirtual）∧ Signature 陈述父类泛型而投影头为裸形` → 拒绝隐藏、保持桥可见。行号会漂移，以锚点名为准。
- [ ] 2.2 协变返回桥零回退（design 决策 2）：`parameter_cast_form == false` 不进新守卫；`Specialized.get`/`Spec.get`/`BR$Base.next` 隐藏行为逐字不变。
- [ ] 2.3 `Spec` 修复后：桥可见、家族 `javac --release 8` 通过、擦除派发 `SPEC.set(String) ran` 与原类一致；`BR$StrBox` 桥可见、`javac` 通过、行为不变（`s`）；接口边既有前置与门 1/门 2 逐字不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（当前主线 296 目标 / 2933 passed；含 `recover-bridge-admission-gates` 全部桥测试、`br_family_negative_shapes_keep_their_refusals`、`the_parameter_cast_admission_walks_the_snapshot_chain_and_respects_its_edges`；已知 flake 家族见 handoff.md，单测复跑两轮判定）；fmt；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成**（含 `--all-features`，29 项 `-A`）；`openspec validate --all --strict`；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 `Spec`/`Specialized`/`BR$StrBox` 与变体三方对照：原 class / 固定 JADX（dev，仅对照不作准入）/ Jarde 重编 `java -Xverify:all` 擦除派发逐路径一致；corpus 双腿扫描——差异应仅"父类边参数收窄桥从隐藏变可见"的形（如 `BR$StrBox`），且逐条确认无行为变化（空体两侧一致），如实记录。
- [ ] 3.3 root 独立复核父类边前置判据、协变返回桥零回退、擦除派发三方行为，更新 EM 账本（桥域）与巡查记录；确认与 `recover-parameterized-superclass-nested-headers`（姊妹根治片）的协调——本片前置在姊妹片落地后自动失效，无需回退。
