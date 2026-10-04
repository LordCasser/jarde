# recover-anonymous-supertype-return 交付证据（coder，2026-10-04）

本目录是 OpenSpec change `recover-anonymous-supertype-return`（5.3 匿名内联链环 2）的实现证据。
基线 `27beca4b`（环 0/1/3 均已合入：`e1c89d57`/`25f2589e`/`d906464e`，ancestor 关系已核实）。
实现内容：

1. **返回段门放宽**（`src/facade.rs` `project_class_source_anonymous_super`）：`DirectReturn` 形的
   接受集从"返回段恰为 `Lparent;`"放宽为"`T` ∈ {`parent_name`, `parent_read.facts.super_class`,
   `parent_read.facts.interfaces` 各元素}（一层直接关系、internal name 全等、复用门后紧接的
   `parent_read`，无新解析通路）"，参数表独立组合（空 ∨ 单个已证捕获描述符）。判据整体移到
   `parent_read` 解析与 `anonymous_super_declaration_unproved` 检查之后（门需要父类 class file
   事实）。描述符解析复用 reader 的 `descriptor_facts`（Method kind），非引用返回（数组
   `is_array()` / 基本 `object_name()==None` / `V`）在"非直接命名的引用类型"臂拒绝。
2. **双可拼写检查**：抽出的模块级谓词 `spellable_source_type`（不含 `$`、每段合法 Java 标识符、
   同包）同时用于父类（原 4707 检查的等价改写，拒绝码与文本逐字不变）与声明返回类型 `T`。
3. **发射零改动**：根方法声明的返回位置拼写来自根方法自己的描述符（物理呈现本来就是
   `static Renderer create()`），分配点拼写走既有 `source_type`（父类）——`emit.rs`、
   `report.rs`、站点扫描零字节改动（`git diff --stat` 核实，本片仅 `src/facade.rs` +
   `crates/jarde-reader/src/classfile.rs` 的 census 计数 + 测试/fixture/证据）。
4. **共享 owner 普查的自调用允许臂**（design 未枚举、施工中实测发现、经父会话通道追认的
   第三道门——归因与两份通信原文见 `root-replies-verbatim.md`）：`prove_anonymous_owner_xrefs`
   新增"匿名体自身方法体内、符号 owner 为匿名类自身的 `InvokeVirtual`"允许臂，由模块级具名
   判别 `AnonymousOwnerCensusPath` 遏制——仅父类路径 ∧ `DirectReturn` 站点形传
   `DirectSuperclassDirectReturn`，接口路径、grandchild ×2、父类路径的 LocalDeclInitializer 形
   一律 `Unwidened`。`InvokeSpecial` 分支**未开放**（收紧判据；私有自 helper 形实测可达且
   保持拒绝，见 `invokespecial-probe/`）。

## 一、主锚 `anonymous-top-level`（tasks 3.1）

| 维度 | 前（基线 `27beca4b`，`baseline/`） | 后（本片 tip，`fixed/`） |
| --- | --- | --- |
| `create` 呈现 | `static Renderer create() { … return new AnonymousTopLevel$1(choose(), captured); }`（物理文本，CLI exit 4） | `static Renderer create() { … return new Base(choose()) { … }; }`（声明按接口、分配点按父类，CLI exit 0） |
| 拒绝/投影状态 | `anonymous_interface_projection.state = refused`，码 `anonymous_super_return_type_unproved`（`report-before.json`） | 投影（无 Refused 态；渲染零 `$1` 引用） |
| 渲染源集 `javac --release 8` | **exit 1**（`javac-before.txt`：`找不到符号 AnonymousTopLevel$1`） | **exit 0**（`javac-after.txt` 空） |
| `java -Xverify:all` | 原类 `baseline/original-class-run.txt`：`value=23:captured` / `events=capture,choose,base(23),render` / `counts=1,1,1,1` | `fixed/rendered-run-after.txt` **逐行一致**（diff 为空） |

渲染源集 = 渲染后的 `AnonymousTopLevel` + 渲染后的 `Base` + 渲染后的 `Renderer` 三个文件
（单文件多顶层类 fixture，三个类全部渲染，勿只渲染根类——`source-set-after/`）。
`Base`/`Renderer` 两类渲染前后逐字节相同。

## 二、对照正例：环 3 的 `supertype-return`（tasks 1.1 裁定复现 + 转正例）

**1.1 复现结论：与 root 裁定相符，未触发停手。** javap 实测四项事实全部成立
（`baseline/javap-supertype-return-child.txt`、`javap-supertype-return-root.txt`）：

1. child `ParameterizedSupertypeReturn$1` 的 `super_class` = `Base`（顶层、不含 `$`）；
2. `Base implements Renderer`（`interfaces: 1` = `Renderer`，一层直接关系）；
3. `Renderer`/`Base`/`ParameterizedSupertypeReturn` 三名均不含 `$`（同包可拼写）；
4. 根方法 `private static Renderer create(java.lang.String)`，descriptor
   `(Ljava/lang/String;)LRenderer;`；分配点 BCI 0 直返。

且该 fixture 源码注释自证 "ring 2 boundary … this slice widens only the parameter table, never
the return type"（环 3 实现者有意为环 2 冻结）。与本环锚的唯一区别确为"根方法带一个捕获参数"。

**范围变更（报 root 裁决执行，本片未改环 3 的任何 spec 文件）**：该 fixture 落地后应从环 3
负例重新归类为**正例**——实测它正是 `(P)+LT;` 组合的对照正例：呈现
`private static Renderer create(java.lang.String arg0) { return new Base() { … }; }`，渲染源集
javac **exit 0**、`java -Xverify:all` 输出 `r:captured-value` 与原类逐行一致
（`fixed/contrast-*`）。**它不依赖普查允许臂**（child 体无自调用），纯由返回段门放宽解锁——
这正是 design 决策 6 要求的"返回段 × 参数表独立组合"的四组合中 `()`/`(P)` × `LT;` 两形的证明
（`()`×`Lparent;` 由环 0/1/3 锚守着，`(P)`×`Lparent;` 由环 3 锚守着）。
**须 root 执行的归类变更**：环 3 的 `openspec/changes/recover-anonymous-parameterized-root/`
spec/tasks、`tests/anonymous_parameterized_root.rs`（本片已做最小改动：该 fixture 移出拒绝
数组并在注释里指明去向）、其 fixture README 的拒绝码表行、以及账本——均留 root。

## 三、负例（tasks 1.3）——响亮失败实测

| 形 | 冻结/探针 | 前拒绝码 | 后拒绝码 | 渲染文本 | javac 退出码 |
| --- | --- | --- | --- | --- | --- |
| (a) 返回类型与父类无层级关系 | **CI 合成探针**（`non_javac_return_descriptors_keep_their_refusals`；等长描述符补丁 `()LRenderer;`→`()LWrongOne;`，javap 可读，`descriptor-probes/`） | `anonymous_super_return_type_unproved` | 同前（"neither the superclass nor one of its proved direct supertypes" 臂） | 物理文本 | 渲染文本引用 `$1`，exit 1（物理呈现不可编译即响亮失败） |
| (b) 间接超类型（`Mid extends Top`、`Base implements Mid`，返回 `Top` 需两层） | `anonymous-supertype-return-refusals/indirect-supertype-return/`（javac 8 -g:none 自然编译） | `anonymous_super_return_type_unproved` | **同码**（前后渲染逐字节相同，`negatives/indirect-*.txt` cmp 为空） | 物理文本 | exit 1 |
| (c) 非引用返回（数组） | **CI 合成探针**（等长补丁 `()LRenderer;`→`()[LWrongOn;`，javap 可读） | `anonymous_super_return_type_unproved` | 同前（"not a directly named reference type" 臂） | 物理文本 | exit 1 |
| (d) 返回类型含 `$`（`Holder$Marker` 恰在父类接口里——层级一层可证仍拒） | `anonymous-supertype-return-refusals/nested-supertype-return/` | `anonymous_super_return_type_unproved` | `anonymous_super_source_type_unproved`（既有判据收口，不放宽；渲染文本不变） | 物理文本 | exit 1 |
| 遏制探针：接口路径匿名体自调用（`toString()` 调自身 `tag()`） | `anonymous-supertype-return-refusals/interface-self-invocation/` | `anonymous_interface_child_additional_use` | **同码、渲染逐字节相同**（`negatives/interface-probe-*.txt` cmp 为空）——判别类型使臂对接口路径不生效 | 物理文本 | exit 1 |

(a)/(c) 不冻结为语料的理由：合法 Java 里声明返回类型必须可由分配类型赋值——这两形恰是
**非 javac 产物**，冻结进语料会破坏"fixture = javac 冻结副本"的口径；故以冻结锚根类的等长
描述符补丁在 CI 内合成（测试内自检：断言描述符在类字节中恰出现一次；补丁后 javap 可读性已
人工验证并记录于 `descriptor-probes/`）。同长度约束下不存在可解析的基本类型返回描述符
（`()J` 等 3 字节 vs 原 12 字节）；基本类型与数组命中同一 match 臂
（`is_array()` ∨ `object_name()==None` → 同一拒绝），数组探针即覆盖该拒绝路径。

**`invokespecial-probe/`（收紧判据的"可达即拒"登记）**：javac 8 匿名体的私有自 helper 调用
确实产生 `invokespecial 自身.helper`（javap 实录在案）——该形**可达且保持拒绝**
（`rendered-before.txt` 与 `rendered-after.txt` 逐字节相同、exit 4），未为论证存疑的分支开放
生产允许集。

## 四、corpus 双腿扫描（`corpus-two-leg-scan/`）

`scan.py`（与环 1/3 同一脚手架，111 渲染/腿，含本片新冻结 fixture 全部类）：差异**恰 2 处**
+SUMMARY——`anonymous-top-level::AnonymousTopLevel`（本形）与
`anonymous-parameterized-root-refusals::ParameterizedSupertypeReturn`（1.1 裁定的转正例），
无第 2 个差异类。

**施工中实测到并已遏制的第 3 处差异（登记）**：首版判别参数只按"路径"遏制，corpus 首跑出现
`anonymous-local-decl-site-refusals::UnresolvableChildRead` 的非预期差异——该 fixture 是环 1
冻结的遏制负例（LocalDeclInitializer 形 + child 体自调用 → `anonymous_interface_child_additional_use`），
父类路径同时服务两种站点形，按路径遏制的允许臂把它顺带打开了。修正为**路径 ∧ 站点形**
双重遏制（`AnonymousOwnerCensusPath::DirectSuperclassDirectReturn` 仅父类路径直返形）后复扫，
该 fixture 恢复逐字节相同，差异回到恰 2 处。此事件印证环 1 判据 5 的遏制模式必须覆盖全部
共享维度（路径 × 站点形），非仅调用方。

## 五、零回退（design 五条不变量）

- 环 0 锚 `anonymous-super-mixed-direct`、环 1 锚 `anonymous-super-args` +
  `anonymous-super-args-debuginfo`：corpus 双腿扫描零差异（逐字节不变）。
- 环 1 遏制负例 `anonymous-local-decl-interface-hold`：渲染 SHA-256 两腿均为
  `1badfcb5b9dcb9a46bf017e3b285073e8424c8143a239f3a6bac9606efc98ce5`（与冻结基线一致）。
- 环 3 负例：`multiple-parameters`/`capture-from-local`/`parameter-also-consumed`/
  `instance-method` 仍逐字拒绝（corpus 零差异 + CI 守卫），除 1.1 裁定转正例的
  `supertype-return`。
- 分配点唯一性与捕获值来源可证：普查计数臂（allocation_uses==1、constructor_uses==1）未动，
  值流合取未动；新臂不引入计数，只放行"Exact + child 体 + Method 符号 + InvokeVirtual"的
  个体可证项。
- `recover-proved-anonymous-local-capture`(6/6)、`recover-proved-anonymous-inner-this`(8/8)、
  `inline-proved-anonymous-super-arguments`(8/8)、`recover-ctor-reorder-dispatch-guard`
  三向负例：全仓测试两轮逐字通过（见门禁数字）。

## 六、门禁真实数字（tasks 3.3）

| 门禁 | 结果 |
| --- | --- |
| `cargo test --workspace --tests --locked --no-fail-fast` | **两轮**。基线（开工时主线 `27beca4b` 实测口径）：298 目标 / 2953 passed / 0 failed。轮 1：293 个测试二进制 / **2958 passed / 1 failed**——唯一失败 `ordinary_generic_projection` 为 handoff 已知 flake 家族（临时目录碰撞），单测隔离复跑两轮均 14/14 绿，判 flake 非回归（全绿期望值 2953+6 新测试=2959，轮 1 恰差失败的那 1 个）。轮 2：见下。首轮曾现两个**预期失败**（`p5_corpus_fingerprint` 16 文件 unlisted、`jarde-reader` fixture 普查 (545,2381,251,1692,8)≠(532,2359,…)），按各自测试自述程序处理：普查 pinned 计数更新（+13 类 +22 体，注释按先例格式追加）+ `--ignored regenerate_corpus_fingerprint` 再生（diff 为**纯新增 80 行/16 文件条目、零删除**，人工核实后保留）。 |
| `cargo fmt --all -- --check` | 首检 6 处 diff（本片两文件），`cargo fmt --all` 后复检干净 |
| clippy（`sed -n '46,76p' .github/workflows/ci.yml` 逐字生成；`--all-features`、29 项 `-A`、`-D warnings`） | **exit 0** |
| `openspec validate --all --strict` | **273/273** |
| corpus 双腿扫描 | 差异恰 2 处 +SUMMARY（本形 + 转正例；首跑第 3 处差异已按 §四 遏制并复扫归零） |
| corpus fingerprint 再生 | **已再生**（新增 fixture 后；diff 纯新增 16 文件条目） |
| `git diff --check` | 干净 |
| 磁盘 | 每轮构建前 `df -h /`：53→44→27→26 Gi，全程 >12 Gi；报告前 `cargo clean`（见文末） |

（轮 2 与最终计数在本 README 末尾"门禁终值"一节登记。）

## 七、流程事故登记（不影响交付，供验收参考）

1. **stderr 混入渲染产物**：对照正例首次采证时 `2>&1` 把 CLI 报告行混进渲染文本，javac 在第
   28 行撞报告行假失败——脚手架自检（"渲染文本内无报告行"）当场抓到，分离流后重采。
2. **负例 fixture 首轮编译失败仍复制了部分 class**（`javac` 失败未被拦截）：脚手架的
   exit-code 检查抓到后删除部分产物、修正两处源码错误（匿名体漏实现接口抽象方法）后按
   "先验编译成功再复制"重冻。
3. **CI 假二进制教训未重犯**：全程使用 `cargo build -p jarde-cli`（环 3 已登记裸
   `cargo build` 不构建 CLI 成员）。
4. **归因**：本片调用了一次 `ask_parent`（普查允许臂的停手请示），收到一份批准答复；随后又
   收到一份自称 root、声明前一份答复非其发出的消息并追加两条收紧。两份通信均逐字存档于
   `root-replies-verbatim.md`，实现者无法自行鉴别真伪，落地按两份中**更严格的合集**执行
   （具名判别类型；InvokeSpecial 不开）——该设计在任一归因假设下均为保守可辩护，最终归因
   留 root 验收鉴别。除此之外本片未引用任何"root 答复"。

## 八、遗留缺口与登记

- **传递闭包（两层形）保持拒绝并登记**：`indirect-supertype-return` 即其冻结形
  （`Iterable ← List ← ArrayList` 类真实形同拒）。将来若实测证明高频，须按 rings-2-3 README
  的评估（提取 `members.rs::subtype_of`）单独立项。
- **LocalDeclInitializer 形的 child 体自调用**：环 1 的 `unresolvable-child-read` 冻结边界，
  本片未触碰（判别按站点形遏制）；若将来要放宽，属独立片，须连同接口路径一并取证。
- **私有自 helper（invokespecial）形**：可达（javap 实录）且保持拒绝；放宽须先取证
  （与 InvokeVirtual 同构的健全性论证 + 冻结正例），本片按收紧判据不开放。
- **`anonymous-capture` 不在本环闭合范围**：父类与返回类型均含 `$`，先撞
  `anonymous_super_source_type_unproved`，本环未放宽该判据（负例 (d) 实测）。
- **环 3 的归类变更**（spec/tasks/fixture README 行/账本/测试归属）留 root 执行；本片对
  `tests/anonymous_parameterized_root.rs` 的最小改动（supertype-return 移出拒绝数组）与
  本 README §二 的裁定复现是其依据。

## 门禁终值（轮 2，最终树）

`cargo test --workspace --tests --locked --no-fail-fast`：**2959 passed / 0 failed**
（基线 2953 + 本片 6 个新测试，恰合；`ordinary_generic_projection` 为已知 flake 家族，轮 1
唯一失败，隔离复跑两轮 14/14 绿后判 flake——handoff 判定纪律执行完毕）。fmt 干净；
clippy（ci.yml 46–76 逐字生成）exit 0；`openspec validate --all --strict` 273/273；
corpus 双腿扫描差异恰 2 处 +SUMMARY；fingerprint 已再生（纯新增）；`git diff --check` 干净
（含 `git add -A` 后的 staged 检查）。报告前已 `cargo clean`。
