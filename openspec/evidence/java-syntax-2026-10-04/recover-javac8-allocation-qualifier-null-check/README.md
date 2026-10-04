# recover-javac8-allocation-qualifier-null-check —— 实现与取证记录（实现者，2026-10-04）

实现 change `recover-javac8-allocation-qualifier-null-check` 的落地记录。**结论：分配限定符形
（`new Outer().new Inner(…)`）在真 javac 8 下已恢复——主锚 `N1` 族源码区 quotes 13→0、
`new N1().new Inner(3).total()` 与同族参数限定符 `arg1.new Inner(9).total()` 都折叠；无实参
对照正例 `Pod`（`new Pod().new Nut().mark()`）双腿折叠；corpus 558 类双二进制逐类渲染
零差异；本片五个 fixture 族之外全部逐字节零回退。**

## 一、任务 1.1 插桩定论（第一道门）

按 tasks 1.1 以 CF11DBG 式临时插桩（`ALLOCDBG` 环境变量门控的 `eprintln!`，验收前移除、
`grep -rn ALLOCDBG crates/ src/ tests/` = 0）对冻结真 javac 8 的 `N1.main` 渲染转录：

- **转录**：`results/allocdbg-before-fix-transcript-dedup.txt`（去重）、
  `results/allocdbg-before-fix-full-stderr.txt`（全量 stderr）；
  修复后对照：`results/allocdbg-after-fix-transcript.txt`。
- **第一道门 = `init.rs` 的 `verify` 实例读者门（读者门，687 一带）**。每一次 walk 中，
  嵌套分配站点（BCI 16，`new N1`）都先在读者门失败：
  `readers=[23] written=[] producers inst@19 inst@20` → `written.is_empty()` → 694 模板的
  shape 错误（"…read only by instructions this build quotes (BCIs 23)…"，与 root 实测表层
  诊断逐字一致）。`verify_member` 只在读者门失败**之后**才被到达（walk B，带 member
  targets），且因 `nested_sites=[]`（嵌套站点未证明）在分配限定符臂被跳过。
- **root 的开放问题 (1)（为何 `outside_readers` 只返回 [23] 而非 [23,29]）由 SSA 值级
  dump 定论**（转录内 `ALLOCDBG ssa head=16` 行）：
  `@29 invokespecial N1$Inner.<init>` 的 outer 操作数是 `Stack(3)vValueId(11)=i23` ——
  **舞蹈 `dup@23` 的第一写入**，不是构造产物 `v10=i20`。即舞蹈**重定义了实例值**：dup 读
  v10（def=i20 ∈ produced_by，故 23 是读者），而 29 读 v11（def=i23 ∉ produced_by，故 29
  不是读者）。
- **由此证伪 design「关键未知」段的读码推断**（"dup(23) 使外层实例有 2 个 use，故
  `single_use_at` 应失败"）：插桩实测 `physical_outer uses=[Some(29)]`、
  `single_use_at=true` —— **方向 B（放宽 `single_use_at`）不需要，该谓词语义零改动**。
  成员臂失败的真因是两个：(a) 嵌套站点未证明（第一道门）；(b) `is_the_instance` 的身份集
  不含 v11 的定义点（舞蹈重定义）。

## 二、实现落点（方向 A + 身份集扩展；仅 `crates/jarde-java/src/init.rs`）

1. **尾部三元组识别 helper `discarded_null_check_tail`**（新私有函数）：站点构造器调用之后
   块内紧邻的三条 `[dup, check, pop]`——dup 须 `Operation::Duplicate` 且读取本站点实例
   （`is_the_instance`）；check 须 `Operation::Invoke` 且过共享谓词
   `facts::is_discarded_null_check`（**决策 2 复用，无第二套拼写判据**）；pop 须 opcode
   0x57 且其读取值 == check 的写入值、两者各单次使用（`single_use_at`）——**"被丢弃"由
   SSA 单次使用事实证明，不靠拼写**。
2. **`produced_by` 扩展**：三元组存在时并入站点自有（与既有先例"构造器自有 dup 不是读者"
   同构，`init.rs:676-678` 注释原文）→ 读者门变为 `readers=[29] written=[29]`。
3. **`Site.instance` 身份集**（新字段）：扩展后的 produced_by 随站点携带；两处消费方切换：
   - `verify` 的嵌套实参检查（"the value is one of the arguments"）用 `nested.instance`；
   - `verify_member` 分配限定符臂的 `is_the_instance(ssa, physical_outer, nested.instance)`
     （v11 def@23 ∈ 扩展集 ✓），`after_nested` 从 `instance.last()`（舞蹈 pop）之后起算，
     使 `member_ordinary_arguments` 的实参窗口不含舞蹈。
4. **决策 3（无版本判据）**：diff 无 `java_release`/`major_version`（自查 grep = 0）。
5. **核心不变量**：`single_use_at` 谓词本体零改动；读者门"恰一个 Java 拼写位"
   （`readers.len() != 1`）零改动；舞蹈的 dup 所重定义的值（v11）自身仍受
   `single_use_at(physical_outer, at)` 约束（uses=[29]）。

## 三、验证记录（门禁真实数字）

| 门禁 | 结果 |
| --- | --- |
| `cargo test --workspace --tests --locked --no-fail-fast` | 见交付报告（后台全量两轮 + flake 家族单测复跑两轮判定；基线 301 目标/2970 passed + 本片 6 个新测试） |
| `cargo fmt --all -- --check` | 干净 |
| clippy（ci.yml 46–76 逐字生成） | 见交付报告 |
| `openspec validate --all --strict` | 见交付报告 |
| `git diff --check` | 干净 |
| corpus fingerprint 再生 | `regenerate_corpus_fingerprint` 通过；`corpus_files_match_the_recorded_fingerprint` ok |
| fixture 计数点（`classfile.rs` repository_class_fixtures…） | 新增 11 类/30 体后按该测试既定程序实测更新为 `(569, 2453, 251, 1763, 8)`，附注释行 |

### 主锚与对照

- **主锚（真 javac 8 `N1` 族）**：`results/render-before-real8-N1.txt`（quotes=13、main 全
  引注、use 整方法 not recovered）→ `results/render-after-real8-N1.txt`（quotes=0，
  `println(new N1().new Inner(3).total())`、`return arg1.new Inner(9).total()`、
  `return new Inner(arg1)` 三形齐折）。**口径**：本目录渲染输出无 JSON 报告尾，源码区口径
  与全文件 `grep -c '@bytecode'` 相同，且每个计数前已核对 jarde 自述头。
- **javac 23 腿零回退（逐字节）**：`render-baseline-javac23-N1.txt` 与
  `render-after-javac23-N1.txt` 逐字节相同（3087 bytes）。
- **双腿 javap 对照**：`javap-real8-N1.txt`（main 的舞蹈 23/24/27、Stat.use 的实参窗口舞蹈
  5/6/9）vs `javap-javac23-N1.txt`（两处均无检查）；Pod 同理。
- **对照正例（无实参形，design Open Question 3）**：`Pod`（标识符/形态与锚不同）——
  `javap-real8-Pod.txt` 显示同样舞蹈（14/15/18），`render-after-real8-Pod.txt` /
  `render-after-javac23-Pod.txt` 双腿 quotes=0、折叠 `new Pod().new Nut().mark()`。
- **行为三方（`java -Xverify:all`，逐行）**：`behavior-*.txt`——N1 双腿 `10/7/13`、Pod 双腿
  `27`，原 class 与渲染源重编（`javac --release 8`）输出逐行一致；重编 exit 0。

### 负例

- **负例 C（用户显式 getClass 语句）**：上一片冻结的 `G` 族——本片 corpus 双腿扫描中
  `g-real8`/`javac23-g` 渲染逐字节不变（`zero-regression-13-families.txt`），即"语句保留 +
  仅折叠分配序列内检查"的行为未被本片触碰；其 CI 断言由上一片测试文件继续守卫。
- **负例 D（多处真实消费/无渲染读者）**：
  - `D2`（任务书 D 形：`o = new D2(); o.hit(); o.new In()`）：实测**与上一片负例 C（G）同
    语义**——用户语句保留为语句、成员构造折叠，双腿渲染逐字节不变
    （`render-after-real8-D2.txt`、CI 断言）。即"局部被多处读取"不构成既有拒绝（G 先例）；
    "实例只有一个 Java 拼写位"不变量活在**构造实例**上，不在局部上。
  - `D3`（构造值被丢弃的语句 `new D3().new In();`）：双腿都保持响亮拒绝（quotes=3、
    `render-real8-D3.txt`/`render-javac23-D3.txt`）——**舞蹈自身不收留任何构造**：唯一剩余
    读者是语句 pop（非渲染指令）→ 读者门 `written.is_empty()` 拒绝。该拒绝在本片前后逐字
    节不变。
- **负例 E（check 返回值未被丢弃）**：真 javac 不可表达 → 等长补丁合成探针（上一片先例）。
  - `E1`（帧一致）：`Class c = null;` 使局部 1 为 Class 型，补丁 `pop@17 → astore_1`
    （`e1-kept-patch-deltas.txt`：offset 537，0x57→0x4c，唯一 delta）。补丁后渲染
    （`render-real8-E1-kept-probe.txt`、`report-json-E1-kept-probe.json`）：嵌套站点的拒绝
    **恰落在读者门**（"the instance the allocation at BCI 6 builds is read only by
    instructions this build quotes (BCIs 13)"）——即 kept result 使尾部不归站点、dup(13)
    重新成为"非渲染读者"。对照未补丁腿（`render-real8-E1-unpatched.txt`）：同一站点
    PRESENTED（"1 presented as new"）——**补丁与否的 diagnosic 分岔正是尾部归属判据**。
  - N1 主锚的同型补丁（pop@27→astore_1）在更早的既有 IR 帧门响亮拒绝（`ir_frame_inconsistent`，
    局部 1 是 String[] 型）——与上一片负例 E 的记录同因：等长补丁无法既保持帧一致又把
    kept-result 送抵形状门；E1 是帧一致的替代载体。

## 四、corpus 双腿扫描

`results/corpus-dual-leg-scan.py` + `results/corpus-dual-leg-scan-output.txt`：

- **自检先行**：扫描器先证明它能看见锚差异（13→0）且两份渲染都带 jarde 自述头，然后才
  采信零结果（handoff「扫描器的空结果不能自证正确」）。
- **tests/fixtures 全部 558 类**（除本片 fixture 目录）：双二进制逐类渲染 **558 identical /
  0 differing / 0 errors**——无任何 `requireNonNull` 形或参数限定符形差异（停手条件 (e)
  未触发）。
- **本片 fixture 族（族根渲染）**：n1 13→0、pod 8→0（DIFFERS = 本片所修缺口）；d2/d3/
  e1-kept 逐字节 IDENTICAL（其冻结行为未被触碰）。

## 五、零回退（逐字节）

`results/zero-regression-13-families.txt`：8 个 `requireNonNull` fixture 类
（BoundNullLambdaAdaptationProbe、AnonymousMemberBase、matrix 四类）+ `N1x`/`Wrap`/
`G` 双腿 + javac 23 `N1` 腿，共 13 族全部 IDENTICAL。

## 六、范围与已知限制

- 生产改动只在 `crates/jarde-java/src/init.rs`（new@1 读者门侧 + verify_member 分配限定符
  臂的身份集/窗口起点），未触碰 `emit.rs`/`report.rs`/`guard.rs`/`member_inner.rs`
  （停手条件 (d) 未触发）。
- **发现一处既有形状缺口（不属本片，未修）**：子类**构造器**内调用 `access$000`（外类私有
  字段读）时 accessor 证明拒绝（"the member table this run holds is X$Nut's"）→ 整族
  joint fold 回退。取证：初版无实参对照正例（Nut 构造器读 `seed`）的族根渲染在修复前后
  都非零 quotes（修复使其从 13 引注降到 3 引注——嵌套站点+舞蹈不再被引注——但外层站点
  仍被 `jre_new_interleaved_effect` 拒）。该形与 `N1$Inner`（accessor 在普通方法 `total()`
  中，折叠成功）的差异表明缺口在"构造器内 accessor"的投影路径。**本片正例 B 因此改用
  无 accessor 的纯形态**；该缺口留 root 立项（不并入本片账本）。
- 尾部舞蹈归站点自有后，`Site.expression` 仍只含 `[head..=constructor]`（不含舞蹈三条）；
  资源头部规则若以 expression 证明"站点后紧跟 store"不受影响（舞蹈只出现在限定成员构造
  形，其后是成员构造器而非 store）。corpus 零差异佐证。
