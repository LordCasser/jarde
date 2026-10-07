## 1. 取证与基线（root 已完成大半）

- [x] 1.0 双形判别、拼接 exit 1、jadx 双括号解、字节码次序机制——实测归档。（root 已完成）
- [x] 1.1 插桩/读码定位伴生 ctor 渲染的 val$/super 次序逻辑（member_inner.rs vs facade.rs）；转录存证据。→ **落点是 `crates/jarde-java/src/ctor_order.rs::present_prologue_first`（`build.rs:8082` 唯一调用点）**，非 `member_inner.rs`/`facade.rs`（二者拥有分配点匿名投影与伴生实例化文本，不写 ctor 语句序）。证据：[results/01-locate-ordering-logic.md](results/01-locate-ordering-logic.md)。
- [x] 1.2 重验基线：主线二进制渲染 DB（捕获形 exit 1 现状、无捕获健康）；负例探针（super 实参依赖捕获值）现状记录。→ 证据：[results/02-gating-and-baseline.md](results/02-gating-and-baseline.md)。负例实测**不可验证**（JVMS 4.10.1.9 禁 `uninitializedThis` 上的 `getfield`；JVMS 8/23 均 VerifyError；本运行帧阶段 `ir_frame_deferred` 整方法拒绝）——冻结为 `capture-super-arg-probes/read-arg/`；另冻结可验证的实参调用形 `call-arg/`（`super(compute())`，该类只声明构造器）作为参数走的可触发负例。
- [x] 1.3 冻结 fixture：DB（javac23 `--release 8` 腿）+ DB8（真 javac 8 腿——验证两版 val$ 前置次序一致）入 `tests/fixtures/`，README 记编译命令与 SHA。→ `tests/fixtures/proved-java-structure/double-brace-capture/{v23,v8}/` + `freeze.py`（两腿各断言 `putfield val$s` 先于 `invokespecial` 且 `java -Xverify:all` 输出 `2/z`）+ `README.md` + `SHA256SUMS` + `run.sh`。

## 2. 实现

- [x] 2.1 按决策 1 实现重排（pre-super 全捕获赋值 + super 实参无依赖→重排；否则现状）；呈现次序按决策 2。→ 加**安全前置（3）**（实施必要收窄，见 design.md「实现补记」与 spec）：`java/lang/Object.<init>()V` 档逐字保留，新增「类方法表除构造器/类初始化器外不声明成员」档 + 实参 SSA 依赖闭包判定（读被移动字段或含调用即拒）。呈现次序 = 原有 rotation（组保持相对次序、落在 `super()` 之后、实例块语句之前），未改。
- [x] 2.2 无捕获形伴生与宿主调用形零改动。→ CLI 前后 diff：`DB$1` 与宿主 `DB` 逐字节一致（[results/02](results/02-gating-and-baseline.md)）；857 类 fixture 扫描 + 2725 捕获 evidence 扫描：除捕获形伴生外零渲染差异（[results/04](results/04-corpus-scan.md)）。

## 3. 验证与验收

- [x] 3.1 主锚：DB 双形拼接 `javac --release 8` exit 0、行为 `2/z` 逐行一致（含真 8 腿 fixture）。→ 巡逻 jar + 两条冻结腿共 6 组编译/运行对全部 exit 0 / `2/z`（[results/02](results/02-gating-and-baseline.md)）；根测试 `tests/double_brace_capture.rs` 两腿各跑 `javac --release 8` + `java -Xverify:all`。
- [x] 3.2 零回退：无捕获形逐字节不变；负例保持现状；corpus 双腿扫描差异类仅为捕获形伴生（记录数量）。→ 见 [results/04-corpus-scan.md](results/04-corpus-scan.md)（fixtures 857：2 渲染差异 = 新 fixture 两腿的 `DB$2`；evidence 2725：1 渲染差异 = 巡逻 `db.jar::DB$2`；两处控制套件 2/2 与 7+6 绿）。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。→ 见 [results/06-gates.md](results/06-gates.md)（实施者记录；fingerprint 已再生：+17 条目、零改写）。
- [ ] 3.4 root 独立复核：重排条件保守性（数据流证明而非猜测）、零回退实测；关闭 summary.md 登记行。（留 root）
