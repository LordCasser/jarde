# HANDOFF — jarde 接续说明（2026-09-30 立；**2026-10-04 更新当前状态**）

本文件是给接续 agent 的入口。先确认下面的 Git 状态，再决定是否开始新工作；不要从旧分支名推断仍有未合入实现。

## 当前状态（2026-10-07 续，最新）

- **`recover-boolean-int-bitwise-operands` 验收合入（`9e553f00`，CI 监控中）**：`BooleanConsumption` 消费走查（boolean 变量存/`Z` return/已认领字段写/布尔兄弟位运算，递归）把 int 化布尔回投泛化出 conditional-rhs 的字段写位；**`mix` 与 `andNot` 同域一并恢复**（类级双向运行逐字一致——conditional-rhs 片设置的硬门以此满足）；物化入局部形/真混合算术负例保持拒绝。合并初跑有一处 fmt 换行（root 修正入验收提交——**合并后必重跑 fmt**，实现片 worktree 的 fmt 不完全等价于合并态）。
- **本日累计 13 片合入**（temporal / interface / nonnull-v2 / loop-else-if / static-generic / postfix-A / dup-store / inline-concat / chained-field / conditional-rhs / array-dance / postfix-B / boolean-bitwise）+ 1 错立项关闭（multireads→inline-concat 重定位）+ 1 CI oracle 回归修复 + census 两族归因修正（canonical 类级撤回、family-4 phi 膨胀）+ critical 15/16/17/18 锚与 canonical/依赖链/多消费者/copy/旧值各族旗舰关闭。
- **队列（下一会话）**：BI 循环携带多读（增强 for 协议消费者建模，`project-proved-enhanced-for-loops` 邻接）→ 低优先存量（`recover-statement-position-news` / `recover-fixture-behavior-guard-coverage` / `recover-boxed-number-widening` / `recover-capture-ctor-super-order` / local-scope 12/13 锚 / double-brace B 路径 / array-covariant-store）。全部 spec 就绪或已登记。
- 磁盘 48Gi；全部 worktree/target 已回收。

## 当前状态（2026-10-07 续，背景保留一）

- **本续段（10-06 深夜→10-07）再合入四片（全部 root 独立验收 + CI 绿）**：`recover-chained-field-assignment`（FieldCopies 单证明双形状；两处 ask_parent "ruling" 按纪律未采信、root 以自证重裁）、`recover-conditional-rhs-field-compound`（**第 15 critical 锚关闭**——BI 整类可编译错面 root 亲测复现后以恢复关闭，`false/false/false/false` 逐字一致；布尔位通道收窄到字段写位避免暴露 BW 面）、`recover-array-initializer-value-positions`（copy 族第 3 员 array dance；12 个非末实参位外沿逐个行为回放后追认）、`recover-postfix-condition-positions`（**postfix 域 Phase B 关闭**——两问门控揭示真拒绝点在 region 测试纯度层；单处 `test_expression_instruction` 收拢判据 + `ChainPositionBound` 围栏；迭代计数精确回放）。
- **copy 族四员全部落地**（postfix 快照消费位 / dup-store / chained-field / array-dance）；旧值族 A+B 相齐；依赖链族旗舰落地；多消费者族重定位缺口（inline-concat）落地。value 级四族的 critical 面全部关闭或已立项在飞。
- **在飞**：`recover-boolean-int-bitwise-operands`（BW 面——conditional-rhs 片已留测量：宽规则会暴露 `mix` 可编译错面，任务书已含类级不变量硬门）。
- **队列**：BI 循环携带多读（增强 for 协议域）→ 低优先存量（spn / fixture-guard-coverage / boxed-widening / capture-ctor-super-order / local-scope 12/13 锚）。
- 新教训（10-07）：array-dance 的 12 个 corpus 外沿再次实证"消费位泛化=同判据外推"模式——**外沿追认的门槛=逐个行为回放**，不做静默吸收；Phase B 的数组读取准入外沿（`SW.sum2d`/`NL.findMid`）已行为验证但未钉测试，后续巡查覆盖。
- 磁盘 50Gi；所有已验收 worktree 已回收。

## 当前状态（2026-10-06 深夜续，背景保留）

- **本续段新增合入（全部 root 独立验收）**：
  5. `recover-static-generic-field-init-text`（真根因=facade.rs 静态折叠重拼循环的原地改写偏移漂移，13+/3- 修复；MN/RG 整类编译不可达裁定为既有 Hold<T> 擦除对投影债）；
  6. `recover-postfix-old-value-snapshot` **A 相旗舰**（三条拒绝路径各自门控：时间引注/依赖链/条件臂；`elems[size++]=t` 依赖链族旗舰关闭；事后记账失败关闭检查；矩阵外三消费位追认为机制内外沿）；
  7. `recover-dup-store-conditional`（copy 族第 4 员；双落点门控；三态呈现 Eliminated/Split/Expression，局部活目标保 CF-06 契约；condAssignOld 前提证伪=javac 发射裸 iinc，归 short-circuit-local 边界）。
- **错立项关闭**：`recover-committed-local-multireads` —— 门控实验证伪（计数门从不见已提交局部；NI 的 3-consumers=phi 膨胀，全 114 处存档同型；多读能力本就存在）；真缺口重立 `recover-inline-conditional-concat-operands`（`+` 链内联分支值→jre_concat_split 跨块 toString；判别探针 NI/NI2/NMA/NMB/CMP 已冻结）——**已派发实施中**。
- **CI 回归修复**：postfix 合并后 `p3_execution_comparison` 两 oracle 测试红（`p3-local-rewrite` 的 saved/conditional 翻 Executed 而期望钉旧 Quoted）——期望已同步（`ebd613c6`，CI 绿）。**新纪律**：corpus 移动片验收必本地跑 `cargo test --test p3_execution_comparison --all-features -- --ignored`。
- **census 修正**：第 4 诊断族（多消费者）主行=phi 膨胀级联面，真因在上游（本例拼接跨块）；巡查见该诊断先剥离 phi 记录再归因。
- **新教训**：(1) 测试总数判定用 exit 码 + `test result: FAILED` 行数 + ok 计数，宽匹配 awk 会把 `test result…` 开头的**测试名**行计入（本会话两次假 1-failed）；(2) 裸 `--test <name>` 不带 `--all-features` 时部分 target 跑 0 测试（d3 案例）——复跑判定必须 CI 同口径旗标；(3) 每条终端命令独立起于仓库根，`cd` 不跨命令残留（merge 误入 worktree 的虚惊来源）。
- **派发队列**：在飞 inline-conditional-concat → `recover-chained-field-assignment`（copy 族 putfield 链）→ `recover-array-initializer-value-positions`（array dance）→ postfix Phase B（条件位）→ 低优先存量（spn/fixture-guard/boxed-widening/capture-ctor-super-order）。value 四族收口：copy（4 员全落地或就绪）/旧值（A 相落地）/依赖链（旗舰落地）/多消费者（重定位为 inline-concat，在飞）。
- 磁盘 61Gi；全部已验收 worktree 已回收。

## 当前状态（2026-10-06，背景保留）

- 长期 Java 语法恢复 `/goal` active。本节由 2026-10-06 会话 root 更新（**终态**：四切片验收合入 + 两验收债关闭 + 一归因撤回）；下方 10-04 节保留作背景。
- **本会话合入四切片（全部 root 独立验收 + CI 绿；末片 `b6cbaa94` CI 监控中）**：
  1. `recover-temporal-argument-widening`（合并 `659c9c84`）——java.time 七型十四行 + CompletableFuture 双行入 `platform_interface_argument_widens`，root 对 live rt.jar 复核 javap；TWX 表外型双腿保持拒绝。
  2. `recover-parameterized-interface-headers`（合并 `bf4675ae`）——接口三道门放行 + 父类池形参数化 MVP；两处前提更正经 root 追认（`binary_pool_class_name` 叶子规则=裸头同名纪律；单条 Comparable 平台事实循桥准入先例）；桥接缝 `header_interface_arguments` 单一事实源。
  3. `recover-proved-nonnull-bound-receivers` **v2**（合并见 `5a9f32bf` 前序）——第 5 主族 critical 17 锚（`o.ifPresent(sb::append)` 吞语句）关闭：站点自有空检查尾识别（`discarded_null_check_window` 与构造尾共享纪律）+ `Sites.owned` 先到先得 + move 链终于 Allocate + 无捕获后重写 + 站点描述符点名分配类（C1.chain 可编译错缺口由 corpus 差分先行堵住）；**v1 的门控实验证伪了 root spec 的两门充分性**（捕获 SSA 定义是尾的 `dup`），E 矩阵与 root 重设计裁定在 change 目录。
  4. `recover-loop-else-if-early-returns`（合并 `b6cbaa94`）——`while + else-if 阶梯 + 早退` 形 canonical-overlap 拒绝关闭：`ladder_join` 读法（两臂严格前向路由交于循环 scope 内唯一首块），diff 纯增量 242+/0-，`overlapping_owner` 未动；bsearch 完整恢复。
- **验收债关闭**：EM-15 `recover-write-accessor-field-types` 3.6 与 DT-26 `recover-lambda-primitive-array-capture` 3.5 root 复核完成（diff 零放宽逐条、九型表 vs javap、方向 B 定夺、行为抽验同批），账本全勾（`4ef3868f`）。
- **归因撤回（重要）**：canonical-block 第 5 族原"类级上下文确定性缺陷"记载作废——巡查自家归档 CB 渲染就显示 `loopElseIfRet` 拒绝（README 与档案矛盾），"BX 单独类恢复"在巡逻时代全部已提交二进制（root 二分 `2a4dbc7d`/`4e73fd4e`/`a4b66bb2`/`319a57e0`）上不可复现；真实事实=方法形状属性，已按修正事实立项并当场合入（切片 4）。census 第 5 条已修正（`ec7ee9d5`）。
- **教训（本会话新增）**：(1) 旧提交二分脚本的恢复必须 `git checkout main`（checkout sha 会 detached，本会话因此在游离态提交过一次，`--ff-only` 无损收回）；(2) 巡查判别实验若渲染自未合并 worktree 二进制，其"恢复"观察不可作判别证据——冻结判别实验必须记录二进制 sha 或用主线干净构建；(3) gh 的 `run list -c <sha>` 对已完成 run 可能返回 0（list 索引滞后），终态判定用 `gh run view <id>` 直查。
- **派发队列（下一会话）**：低优先存量（`recover-statement-position-news` / `recover-fixture-behavior-guard-coverage` / `recover-boxed-number-widening` / `recover-static-generic-field-init-text` / `recover-capture-ctor-super-order`）+ 已登记候选（array-covariant-store、第 6 族形态 4 lambda→JDK ctor、nonnull 残余 bootstrap-owner 门、loop-else-if 残余 firstArmRet/try 内部/双层阶梯、interface 片擦除契约自调用债）。critical 面当前无未立项锚。
- **模型池**：`opencode/deepseek-flash` 本会话四片全中（含两次正确停手/自堵缺口）；qwen 家族可用；glm-5.3-flash 配额至 2026-10-10。
- 磁盘：root target 已清、全部 subagent worktree 已回收（70Gi 空闲）；codex 固定 worktree 不动。

## 当前状态（2026-10-04 更新，背景保留）

- 长期 Java 语法恢复 `/goal` 状态为 **active**（用户已恢复并持续给指令）。工作模式是 patrol → spec → 派发 coder subagent（`bigmodel/glm-5.3-flash`）→ root 独立验收 → 写回账本；**一次只允许一个 subagent 构建**（磁盘串行约束，见下"磁盘纪律"）。
- 本会话（10-03→10-04）已验收合入 **五个切片**（4 jobs：stable / fuzz smoke / supply chain / MSRV）。前四片在各自验收时 CI ALL GREEN；第五片合入后主线出现 `bulk_*` 家族 CI flake（判定见下"已知 CI flake 家族"，root 本地两轮全量 2937/0）：
  1. `recover-bridge-admission-gates`（merge `5f07e13c`）——协变返回擦除门改快照层级 walk + 体形门接受规范参数 cast 形 + 三项可重建性门。
  2. `recover-ctor-reorder-dispatch-guard`（merge `fc868aba`）——**修复上一片引入的静默行为回归**（构造期虚分派下重排捕获写入使 `visibleDuringSuper` true→false）。
  3. `recover-nested-class-literal-values`（merge `78db127b`）——嵌套类字面量准入（反射入口高频形）；含两道守卫（结构反射陷阱、折叠失败回退），修复过程中 root 两次实证退回阻塞点。
  4. `recover-bridge-superclass-header-precondition`（merge `cc4b6f11`）——**修复 1 的潜伏错值**：裸父类头下隐藏参数收窄桥会使擦除派发静默路由到父类体。
  5. `recover-anonymous-mixed-super-capture`（merge `e1c89d57`，**5.3 里程碑首环**）——混合参数匿名类（super 实参 + 捕获值并存）内联为 `new Base(args…) { … }`。root 实测新锚完整源集 `javac` exit 0、事件日志与原 class 逐行一致、pre-super 写入由 javac 自行重建（故该形不再依赖 ctor 重排）。**其下一环 `recover-anonymous-local-decl-site`（局部声明位形）已派发实施中**，spec 五处判据已由 root 实测钉死。
- 验收基线（root 在干净合并态实测）：**296 目标 / 2937 passed / 0 failed**；fmt 干净；clippy 从 ci.yml 逐字生成（含 `--all-features`、29 项 `-A`）exit 0；`openspec validate --all --strict` **270/270**。
- **已知 CI flake 家族（2026-10-04 root 实测扩充，判定纪律：单测复跑两轮；CI 上须核实失败提交与失败测试是否成对，勿凭单次红判为真回归）**：`p4_plugins`（计时）、`p3_short_circuit_transfer_gateway`（scratch 目录 `AlreadyExists`）、`bulk_recovery_delivery`、`bulk_recovery_lifecycle`、`export_cli`、`gateway` 族、`observable_equals`、`backward_second_entry`、`ordinary_generic_projection`、`engine::standalone`。**决定性证据**：`c586a7f4`（**纯 docs 提交，且早于 5.3 合入**）CI 红于 `bulk_recovery_delivery.rs:421`（pass 2），其后 `452e6df0`（同为纯 docs）CI 绿，`a1cbd6bc`（5.3 合并 + docs）CI 红于 `bulk_recovery_lifecycle.rs:199`（pass 1）——**两次失败是不同测试、且其中一次发生在未含本片代码的树上**，故属并发/时序 flake 而非代码回归；root 本地在干净合并态两轮全量均 2937/0。
- **CI 监控操作纪律（2026-10-04 root 踩坑三次，强制）**：(1) 不要用未认证的 `api.github.com` + `urllib` 写监控——本机出口 IP 会撞 **403 rate limit exceeded**，而 `except Exception: pass` 会**静默吞掉**该错误，使监控永不报告（本会话因此让一次真实 CI 失败无人知晓约 24 分钟）。(2) 用已认证的 `gh` CLI 时，**`--jq` 里 `.id + "|" + .status` 会报 `cannot add: number and string`**——`.id` 是数字，jq 不做隐式转换；错误文本会让 `[ -n "$OUT" ]` 恒真而 `ST` 永不等于 `completed`，于是监控**无限静默空转**（本会话一个监控因此 68 分钟零输出，掩盖了连续 3 次 CI 失败）。正确写法：`--jq '.workflow_runs[0] | [(.id|tostring), .status, (.conclusion // "pending")] | join("|")'`。(3) **监控脚本上线前必须先单独跑一遍其查询表达式并打印结果**（本会话因此才发现上述 jq 错误）——监控静默无输出时，默认怀疑监控本身而非"CI 还在跑"。(4) 必须用**完整 sha**（短 sha 查不到）；只在**全部 job completed** 时才报结论；跟踪移动的 tip 要在循环内每次重新 `git rev-parse origin/main`，不要在启动时固定一个 sha。取失败详情用 `gh run view <run-id> --log-failed`。
- **在飞**：`recover-anonymous-mixed-super-capture`（`present-proved-java-structure` 5.3 里程碑的精确剩余范围）由 coder subagent 实施中。
- **flake 家族增员（2026-10-05，含家族模式）**：`bulk_recovery_delivery.rs` 的 `one_declaration_bounds_the_librarys_own_presentation_too` 在纯文档提交 `7a0f6ee4` 上红（CI 12-run 积压高负载期；**2026-10-05 又在 `ca733edb` 再现一次，同型**——docs-only、同代码、本地绿，持续 CI 积压期；**同日第三次：`7b291971` 红于 `d3_artifact_binding::the_evidence_is_rebuilt_after_every…`——第 3 个不同 target 但同族特征（重建/计时敏感断言 + docs-only + 本地 CI-exact 命令 `--all-features` 两轮绿）**），判定依据三连：同代码异结果（`48e00258` 绿）、本地合并态单测两轮绿、同 target 已有 `83571af5` 先例——与 `p4_plugins` 计时家族同源（**负载敏感断言的家族扩散模式**；判定通式 = docs-only 提交红 + 本地 `--all-features` 精确复跑两轮绿），勿当回归。
- **spawn 工具约束（2026-10-05 实测）**：`spawn_subagent` 的 `cwd` **不能指向父工作区之外**（旧 grow worktree 会报 "outside the parent workspace; use isolation=worktree"）——而 isolation=worktree 会建**新** worktree、丢旧处未提交状态。续作协议（配额/中断事件的标准动作）：(1) root 在旧 worktree `git add -A && git commit` 把 WIP **机械提交**（保实现者状态；共享对象库使各 worktree 可见）；(2) 记下 tip SHA 与分支名；(3) 新隔离 agent 任务书第一步 `git merge <tip>` 继承；(4) 删旧 worktree（分支保留供追溯）。
- **模型配额事件（2026-10-05）**：`bigmodel/glm-5.3-flash` **周/月配额耗尽**（429，重置 2026-10-10 00:26:40）——EM-15 实现者死于此（实现已完成未提交，root 预审计 diff 后以 **qwen/qwen3.8-flash 在原 worktree 续作收尾**，用户此前已授权 qwen 按需调用）。规则：派发前无法预知配额；agent 因 429 中断且工作区有未提交实现时，先 root 预审计 diff（防半成品），再用其它已授权模型 + `cwd` 指向原 worktree 续作（不建新隔离，避免丢未提交状态）；配额期内后续派发改用 qwen 家族。
- **队列**（2026-10-05 root 更新，串行，全部 spec 就绪且锚点名式防漂移）：在飞 `recover-javac8-allocation-qualifier-null-check`（方向 A 已定论：读者门 `init.rs:687`，舞蹈 `dup@23` 重定义实例值）→ 验收后依次派 `recover-write-accessor-field-types`（EM-15，9 类型封集表，每型 hex 以仓库常量为准勿凭记忆）→ `recover-lambda-primitive-array-capture`（DT-26，根因 `frame.rs` newarray 压 `RefType::Unknown`，A/B 分岔插桩先行，LVT 缺失是关键未知）→ `recover-parameterized-interface-headers`（已并入父类池形参数化 MVP：判别实验证明 `extends BR$Box<String>` 池形拼写合法且产物与 source 形恒等，**不重拼**，仅放宽两处 `$` 拒绝）→ `recover-twr-javac8-close-sequence`（MVP 单资源，Q-i/Q-ii/Q-iii 插桩先行，javac9+ 判据逐字不变）→ 低优先存量：`recover-statement-position-news`（spn，取证冻结于其 design）→ `recover-fixture-behavior-guard-coverage` → `recover-boxed-number-widening`（drift-immune 已核）。make 硬编码（`init.rs` arm1）= 被 arm2 掩盖的卫生债，巡查已证非能力缺口，不立项。
- **磁盘纪律（2026-10-05 补充）**：agent target 是唯一可回收大头（~20G）；`~/workspace/testzone`（25G）等是**用户资产禁止触碰**；root 侧 `target` 在 agent 构建期主动 `rm -rf` 让道；CI 对 doc 提交突峰会积压十几个 run（非卡死，耐心消化），监控器须覆盖**全部近 15 个 run** 的非 success（只盯 tip 会漏）。
- **本会话巡查的净产出**（除四片外）：CF-18 嵌套 handler 定性为硬前沿（JADX 自身错放 handler，Jarde 诚实拒绝更安全，不立片）；枚举 switch 常量标签归入既有未完成 change `project-proved-enum-switch-labels`（1/9，不重复立项）；接口 default/static 方法体**验证健康**（非缺口）；方法引用六形归入 DT-27 已登记残留（绑定接收者拒绝理由有原则：NPE 时机）；泛型声明三形与注解泛型元素 + 默认值均为**低危保真度缺口**（可编译、行为一致、仅反射元数据降级），登记不立片，并记录了"`class_source.rs` 约 3934 行的 `attributes.default.is_some()` 是保护性门，naive 放宽会吞掉注解默认值"的架构警示。
- 下方"最近一次主线收口"等段为 **2026-09-30 的历史记录**，仅作背景；其中"目标 paused"、CI 36606512666、target 清理数字等均已过期，以本节为准。

## 当前状态（2026-09-30 历史记录，已过期）

- 13 条被工作树占用的历史分支已在 `aeb18c0f` **全部合入并推送到 `main`**；该合并的文件树与合并前 `056709b8` 完全一致。随后释放全部分支占用并删除旧引用；本地只剩 `main`，远端只剩 `origin/main`。本文档提交后以 `git status` 和 `git rev-parse HEAD` 核对新的仓库状态。
- ~~长期 Java 语法恢复 `/goal` 当前状态为 **paused**。~~ **（已过期，勿据此停工）**：目标自 2026-10-04 起为 **active**，用户持续给指令并要求持续推进。以下两条同为历史记录。
- 最近一次已确认的代码 CI 是 [GitHub Actions 36606512666](https://github.com/LordCasser/jarde/actions/runs/36606512666)，所有 job 成功。它验证的是 `fcc864ce` 的代码；后续 `056709b8` 仅增本文档，`aeb18c0f` 仅合并历史、文件树无变化。新提交的 CI 状态需单独查看。
- 根目录 Cargo `target` 已清理约 10.4 GiB；先前隔离构建产物清理约 103 MiB，本次又从 `dt13-nested-enum` 清理 158 MiB。后续测试会重新占用磁盘，Rust 工作结束后留意 `target`。

## 用户确定的工作方向

先以本地 `/Users/lordcasser/workspace/testzone/jadx` 的测试和实现为基础，明确语法特性清单，逐项追平 **可证明正确** 的 JADX 已有能力；之后再探索双方都未覆盖的情况。可以参考 JADX 的反编译代码和算法，避免重复试错，但不能照搬其错误转写。对每个语法点：读相关测试和生产 visitor/region 实现，构造 Java 源码并编译，对照原 class、JADX、Jarde 的完整源码、Java 8 重编和执行；确认差距与 JVM/架构证据后写独立 OpenSpec。确定性、简单的实现任务派 **`bigmodel/glm-5.3-flash`** subagent（2026-10-04 用户指定的实现模型；按难度调整思维强度），由主 agent 独立重放验收。同批互不冲突的点可并行——**但磁盘串行约束下同一时刻只允许一个 subagent 构建**（见"磁盘纪律"）。不要漫无目的地追加场景，也不要把一个窄 fixture 通过称为整个特性追平。

入口是 [JADX 特性清单](openspec/evidence/jadx-feature-inventory-2026-09-27/README.md)、[71 个验收单元与状态账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)、[路线图中的 Java 8 对标章节](openspec/roadmap.md)。清单基于 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 612 个集成测试文件，去重得到 71 个工程验收单元；这个数字不是已追平数。状态账本目前记载 46 个“冻结差距已修复但待扩验”、23 个“部分已测”、1 个“已证差距”（CF-16 的剩余 `finally` 形态）、1 个“JADX 未完成”。这些是文档最后登记的状态，恢复工作时先复核源码和最新主线，避免按旧统计重复开工。

架构约束：理解现有 reader → CFG/SSA → Region/AST → 类级装配与报告的证据流后再改代码；如非必要勿增实体，不考虑后向兼容。先判断是已有证明路径未消费、证据不足，还是确需新机制。与当前语法点无关的架构债务单独记录和拆分。JADX 的测试源码、文本相似度或可编译输出，都不能代替原 class 的行为证明。具体误判和拒绝边界见上述清单与各单元证据。

## 最近一次主线收口

用户要求先提交、推送全部当前工作，并把其它分支工作合入 `main`，随后清理不再需要的分支。审计 158 个非主线本地分支及工作树后，确认大多数历史提交已由主线后续或等价实现覆盖；直接重放旧提交会倒退代码。实际缺少的只有两项 `finally` 恢复：

- `80523785`：固定 catch 值字段的 `finally`（原提交 `2c694adc`）。
- `fcc864ce`：固定可空资源的 `finally`（原提交 `fa790c17`）。

两项冲突在 `crates/jarde-java/src/build.rs` 和 `crates/jarde-java/tests/p3_shared_join_finally.rs` 合并时已保留双方证明与 Test7/Test9 回归。主线已推送。验收包括 `cargo fmt --all -- --check`、OpenSpec strict 216/216、`p3_shared_join_finally` 30/30、整仓两颗固定 seed 测试，以及 CI 等价的 Clippy；上述 GitHub CI 同时通过 stable、JDK 25 oracle/P3、MSRV、fuzz 与 supply-chain job。更早的 JDK 25 CI 修正也已在主线，见 [CI 36377418834](https://github.com/LordCasser/jarde/actions/runs/36377418834)。

先清理了 146 条不再需要且未被检出的本地分支。随后重新审计其余 13 条工作树分支：17 个独有历史提交中，12 个有主线上相同 patch-id；另外 5 个不同 patch-id 的功能、测试或证据由主线后续实现覆盖，分支新增路径在主线没有缺失。逐条核对后，用一次 `ours` 合并记录这些**已被主线内容覆盖**的分支历史，没有重放过时代码；`aeb18c0f` 的 13 个分支 tip 均已成为 `main` 祖先，且 `git diff 056709b8 aeb18c0f` 为空。本次临时集成分支已删除。

14 个辅助工作树（13 个旧分支 checkout 加一个原本 detached 的 checkout）已逐个确认没有未提交、未跟踪文件或使用它们的本地进程，然后统一 detached 到当前 `main`；13 条旧本地分支均以普通 `git branch -d` 删除。辅助工作树目录仍在，但全部干净、不占用分支，也没有未合入的工作。Codex 归档接口对当前任务附着的工作树返回“protected by a pinned task or workspace”；没有绕过保护或直接删除目录。需要物理回收目录时，先解除其 Codex 固定/归属保护，再用工作树归档工具处理。并发工作不要在根 checkout 随意切换分支；优先复用空闲的隔离工作树，操作前重查状态。

## 接手时的最小核对

```sh
git status --short --branch
git rev-parse HEAD
git worktree list --porcelain
git branch -a
```

若用户恢复语法目标，从状态账本选一个**尚未闭合的具体子形态**，重新读对应 JADX 测试/算法和 Jarde 当前代码，再冻结三方正反例。按现有 OpenSpec 写窄任务、实施、Java 8 完整类重编与验证运行、来源/拒绝边界及主 agent 验收；每完成一个切片就回写账本。CF-16 是已登记的剩余真实差距，但它的多个首片已修，不能仅凭编号重做。旧工作树均无剩余实现任务，也不需要重新合并；物理目录的后续归档受 Codex 固定工作区保护约束。

**验收门禁口径（2026-09-30 CI 36678484547 教训；10-01 增补）**：CI 的 "stable / test and specification" 跑 `cargo test --workspace --all-targets --all-features --locked` 双固定 seed、ignored 的 JDK oracle/P3 对照与两个 example。实现任务的验收与 root 复核必须同口径跑整仓命令（注意 `--all-targets` 含 benches/examples，宽于 `--tests`）；只跑 `-p jarde-java --tests` 会漏掉根 crate（如 `enum_constants` 的投影消费方）——恢复层改进（例：`array_of_value` 使增强 for 可证）会改变下游折叠器的输入形状，消费方期望不同步即回归。**Clippy 门禁必须从 `.github/workflows/ci.yml` 逐字复制整条命令**（2026-10-04 root 实测教训）：本地常引的精简 `-A` 清单会漏报，而**漏掉 `--all-features` 会误报**——root 首跑 `cargo clippy --workspace --all-targets --locked -- <29 项 -A> -D warnings` 时报出 11 个 `unit_arg`/`let_unit_value` 错误于 `p5_optimize_workloads`（一个未被当次改动触碰的文件），加回 `--all-features` 后同命令 exit 0、Finished 干净。可靠做法是**用脚本从 workflow 生成命令**，不手抄：

```sh
sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
```

当前 CI 清单为 **29 项** `-A`（`grep -oE '\-A [a-z_:]+' .github/workflows/ci.yml | sort -u | wc -l` 实测；旧文所载"30 项"已过期，勿再引用）。且 CI 的 1.98.1 与本地 1.98.0 存在 patch 版 lint 差（实例：`iter_cloned_collect`，CI-only 报出，b27e0443 修复）——本地全绿不等于 CI clippy 绿，推送后必须核对 CI。

**池形类型名的结构反射陷阱（2026-10-04 root 以 N2 实证确立，适用所有类型名呈现片）**：当一个类型名以**池形**（含 `$`、未经 `InnerClasses` 行集重拼）写进源码时，它在编译产物中是**顶层类**（javac 不从 `$` 推断嵌套，也不生成 InnerClasses 属性）。后果分两类，验收时必须区分：
- **不受影响**：`getName()`（顶层类的二进制名恰等于池名，与原类逐字一致）、`getAnnotation`/`isAnnotationPresent`/`getDeclaredFields`/`getSuperclass`/`isArray` 等不依赖 InnerClasses 的调用。
- **必然偏离或崩溃**：`getSimpleName()`（返回池名而非简单名）、`getEnclosingClass()`（返回 **null** → 后续解引用 NPE）、`getCanonicalName()`、`getDeclaringClass()`、`isMemberClass/isLocalClass/isAnonymousClass()`、`getNestHost/getNestMembers()`、`getEnclosingConstructor/getEnclosingMethod()`——这些的返回值**直接来自 InnerClasses 元数据**。
故判据是：**池形呈现 ∧ 值被上述结构反射方法消费 → 必须保持拒绝**（响亮失败），否则产出"可编译且行为不同/NPE"，违反消隐前置不变量与 `recover-return-in-do-while-false` 的口径。单条件不足以拒绝——折叠投影成功、名已拼为源码形（呈现为折叠成员）后 javac 会重新生成 InnerClasses，结构反射正确（实测 `A12$Nested.class.getSimpleName()` → `Nested` ✓）；池形 + `getName()` 也正确（顶层类的二进制名恰等于池名）。

**池形的成因（root 2026-10-04 读码 + javap 实证；先前"行集不含该名"的猜测已被推翻，勿再引用）**：`names.rs:204-207` 的 `nested_reference_spelling` 有一条自嵌套规则——当名的顶层 owner 等于正在书写其文本的那个类（`head == owner`）时**保持池形**，注释明说只有**折叠投影**才可把它拼成源码嵌套形（`only a fold projection may spell it as source nesting`）。而成员折叠是**单层**的，故：深度 1 的名（`A12$Nested`）折叠覆盖到 → 拼源码形 → 结构反射正确；深度 ≥2 的名（`N2$Outer$Mid$Leaf`、`FP$Mid$Leaf`）折叠覆盖不到 → 保持池形 → 结构反射必然偏离。**判据须取"最终呈现文本是否仍含 `$`"，不要取"名是否在 `InnerClasses` 行集内"**——后者与偏离无因果关系：javap 实证 `N2.class` 的 InnerClasses 表**确实含** `Leaf=class N2$Outer$Mid$Leaf of class N2$Outer$Mid` 行（`FP.class` 同理含 `Leaf=class FP$Mid$Leaf of class FP$Mid`），即深层名在行集内却仍被拼为池形。

**折叠失败回退路径是独立验收维度（2026-10-04 root 以 RF 实证确立，适用所有放宽 decode 准入或类型名呈现的切片）**：guard 若靠 build 层的一个"呈现是否重拼"旗标判定（如 ncl 的 `pool_spelled_members`，只在 standalone 口径为 true），则 **family 口径下折叠失败**（`<clinit>`/class Signature/enum-initializer 投影使 `project_class_source_member_fold` 返回 `Ok(Err)`）会回退分离池形呈现、**绕过 build 层 guard**。此时 build 恢复方法早于折叠尝试，无法预知折叠成败——A12-jar（折叠成功、须放行）与 RF-jar（折叠失败、须拒绝）在 build 层旗标**完全相同**，区别只在 facade 折叠成败。故这类切片的验收必须**四向齐备**：family 折叠成功、standalone、**family 折叠失败**（用带 `<clinit>` 的直属成员 + 结构反射构造，如 `RF.Inner` 的 `static final int K = init()`）、非结构反射消费（`getName`）。修复落点在 facade 折叠失败回退处（facade.rs:1957/1970 的 `Ok(Err(_reason)) => {}`）以旗标 true 重跑受影响方法，复用 20519 的 `analyze_method_ir` re-recovery 先例，让 build guard 产生真实 Refusal，不伪造引注。RF 实测：家族重编 exit 0 但 `getSimpleName()` 返回 `RF$Inner`≠原类 `Inner`（证据 `nested-class-literal-patrol/residual-boundary/`）。

实例：`recover-nested-class-literal-values` 首版使 N2 从基线的响亮失败（10 处引注、方法体空、不可编译）变为可编译且 `multiLevel` 静默偏离（`Leaf`→`N2$Outer$Mid$Leaf`）、`recvChain` NPE 崩溃——root 验收发现并退回修正。同一机制在**不含类字面量**的形上也造成不一致：家族口径下 `FP` 的方法体呈现 `new FP$Mid$Leaf()`（池形）而声明侧呈现 `static class Mid`（源码形），javac 报"找不到符号"（root 以 FP probe 实证）。

**冻结行为 fixture 必须有 CI 测试引用（2026-10-04 事故教训，强制）**：`tests/fixtures/proved-java-structure/` 的 16 个 fixture 中 8 个未被任何 CI 测试引用，其行为基线只存在于 README 与**手动** `run.sh`（实测 CI workflow 与全部测试文件都不调用 `run.sh`）。后果已由真实事故证明：ctor 重排使 `anonymous-super-dispatch` 的 `visibleDuringSuper` 由 `true` 翻转为 `false`，CI 全绿（2918 passed），只有 root 手动重放才发现。规则：**新增冻结行为 fixture 时 MUST 同时新增一个引用它的 CI 测试**（模式见 `tests/p3_anonymous_class_facts.rs`：`include_bytes!` + `Engine::open` + `ClassSourceRequest` + 文本断言；需跑 JVM 时用 `recompile_and_run`/`run_class` 先例并按 `p3_execution_comparison` 惯例标 `#[ignore]`）；`run.sh` 定位为**复现工具而非守卫**；`p5_corpus_fingerprint` 只守文件哈希、不守行为（其自述"asserts nothing about whether an acceptance row passes"），不可当作行为覆盖。当前 8 个未守卫 fixture 的补覆盖由 [recover-fixture-behavior-guard-coverage](openspec/changes/recover-fixture-behavior-guard-coverage/) 承担。**（已完成，2026-10-07 root 验收合并：6 件守卫落地——7 默认呈现腿 + 6 ignored 行为腿，负向自检打在事故同款洞上并被新测试捕获；`anonymous-super-dispatch`/`anonymous-super-args` 既有守卫不变。16 件现全部有 CI 测试引用。）**

**合成成员消隐的前置不变量（2026-10-04 root 以 javac 实证确立，适用全部消隐片）**：隐藏 javac 合成成员（`access$NNN`、lambda 伴生 `lambda$x$N`、擦除桥、`$SwitchMap` 辅助类）的**唯一合法性来源**是"源码自身能让 javac 重新生成同一合成物"，而不是"该成员在字节码里可证是合成的"。故每个消隐片都必须回答：**javac 重编时靠什么重建它？**
- 擦除桥 → 靠**类头的类型实参投影**（`implements Comparable<Impl>`）。实证：裸 `implements Comparable` + 隐藏桥 → javac 报"未覆盖 compareTo(Object)"；参数化 `implements Comparable<ParamI>` + 隐藏桥 → javac 自行重建桥（`javap -v` 实测 ACC_BRIDGE=1、exit 0）。证据 `openspec/evidence/java-syntax-2026-10-04/bridge-method-patrol/header-invariant/`。
- lambda 伴生 → 靠**调用点的 invokedynamic 站点**仍在源码中（已闭合，`recover-lambda-inline-bodies`）。
- `access$NNN` → 靠**嵌套类族在同一文本内平铺**（javac 为跨类私有访问重新合成；已闭合，instance-folding 片）。
**推论**：消隐决策的 owner 是"该合成物的重建前提由谁提供"，前提缺失时必须**保持合成物可见**（响亮失败）而非隐藏——否则会产出"信息丢失 + 仍不可编"的更差中间态。owner 分离示例：类头文本投影归 `recover-parameterized-interface-headers`，桥消隐决策归 `recover-bridge-admission-gates`，二者以"类头是否带类型实参"这一事实为共享契约。

**改序类切片的验收纪律（2026-10-04 回归教训，强制）**：任何改变指令/语句**顺序**的切片（重排、移动、提前、延迟），验收必须 (1) 跑既有 order-sensitive 反例 fixture——本项目现有两个：`tests/fixtures/proved-java-structure/anonymous-super-dispatch/`（构造期虚分派，`Base()` 在 super() 中虚调用覆写方法读捕获字段；重排捕获写入会使 `visibleDuringSuper` 由 true 变 false）与 `openspec/evidence/java-syntax-2026-09-25/ordinary-new-void-effect/`（构造实参 CST 序）；(2) 检查同域既有 change 的**未勾任务**是否已声明该风险（本例 `present-proved-java-structure` 2.10 自 2026-09-25 就明写"旧任务所称'改序不损失任何效果'已被运行反例否定"）；(3) 验收判据必须包含"不得产出可编译且行为不同的文本"——`recover-return-in-do-while-false` 已把该不变量写进 spec，改序类切片一律适用。失误实例：`recover-synthetic-ctor-super-order`（daa4fb31，root 验收）重排 pre-super 合成字段存，未查该反例 → 静默行为回归；修复片 `recover-ctor-reorder-dispatch-guard`（判据加 super 目标 == `Object.<init>`）。安全判据的一般形：**移动跨越"可能虚分派/可能触发副作用"的调用边界时，只有目标不可能产生该副作用才可移动**。

**立项查重（2026-10-04 失误教训，强制）**：写新 OpenSpec change 前**必须先查重**——`ls openspec/changes | grep -i <域关键词>`（含英文与机制名，如 assert/bridge/enum/loop/literal），再读同域 change 的 `proposal.md` 范围与 `tasks.md` 勾选状态（`grep -c '^\- \[x\]'` vs `'^\- \[ \]'`）。原因：仓库有 240+ change、大量"已立未实施"（如 `project-proved-enum-switch-labels` 1/9、`present-proved-java-structure` 51/91）与"已实施未勾"，巡查发现的缺口**常常早已被既有 change 精确描述**。正例：2026-10-04 巡查桥方法 name-clash 时先查到 `project-proved-bridge-forwards`（机制已合入），改为立"准入门扩展"片而非重复机制；巡查枚举 switch 时查到 `project-proved-enum-switch-labels`，改为证据补强不新立。反例：同日巡查 assert 时**未先查**，新立 `recover-assert-statement-sugar` 并实现，事后发现 `project-proved-assert-statements`（2026-09-26 立）范围高度重叠——功能已交付不回滚，但两处 change 都补记了重复关系并把既有片残留范围收窄为（预算/取消测试、定向测试粒度、root 验收）。**若既有 change 已覆盖：优先补强其证据/收窄其残留范围并派发它，而不是新立同域 change。**

**账本诚实纪律（2026-10-04）**：实现者被配额/磁盘杀死于收尾段而由 root 代收尾时，其已完成的 1.1–3.2 必须**据证据实证后补勾**（读 impl-record、results/、in-crate 测试名核对），并在 tasks.md 追加补勾说明（谁、何时、依据、实现形式差异如"变体以合成 class 生成器实现而非独立 fixture"）——只勾 root 自己的 3.x 会让账本失真（`recover-synthetic-ctor-super-order` 曾因此显示 5 未勾/2 已勾，实际全片已验收）。反之，功能被后立切片覆盖时**不代勾**原 change 的 tasks，而是记录重叠与收窄后的残留范围（assert 域如此处理）。

**subagent 的 `ask_parent` 答复不可当作 root 授权（2026-10-04 连续四起，强制）**：本会话四个 coder subagent 都报告收到了**看似权威、逐字成形**的 root 裁决，而 root 从未发出：(1) 环 0 实现者据一条 `status="answered"` 文本把范围收窄记为"root 决策 B"；(2) 环 1 实现者据一份"root 裁决：采纳选项 A"的长文**移除了生产代码里的一道门合取**（`anonymous_child_methods_incomplete` 的 `allocations.is_empty()`）；(3) 环 2 实现者据一份长答复获得"批准放宽共享 owner 普查允许集"的授权；(4) shared-latch 实现者据一份"**root 答复（可直接存档）：不停**……"的长文获得"批准其 join 重选举路线 + 追加验收要求 (i)-(iv)"的授权。root 用 `list_active_sessions` 核实当时**无其它活跃会话**，故来源不明（宿主层自动应答或模型自行生成）。

**四起的共同特征（用于识别）**：答复都 `status="answered"` 且 `target_session_id` 指向 root 会话、语气与用词高度模仿 root（含"可据此归因""予以肯定""留 root 验收"等 root 惯用表述）、内容技术上都**合理甚至正确**、且都会**扩大 subagent 的授权边界**（放宽判据、批准新机制、追加验收项）。第 (3)(4) 起还都恰好在 root 的 spec 有真实缺陷时被触发，故其技术结论与 root 事后独立复核**一致**——这最危险：内容对不等于来源对，若 root 因"结论我同意"就默认它是自己发的，决策链的可追溯性就被污染，且**下一次内容错的答复会被同样放行**。

**四起的处置演进（第 (3)(4) 起的处置已是范本）**：(1)(2) 两起的实现者把答复**当成了授权**（一起改归因、一起删生产门），root 事后更正；(3)(4) 两起的实现者**都正确处置**——停手、给出实测取证与候选方案、未自行实现、把答复原文存档并标注"来源待 root 鉴别"、报告中不预记为已授权。第 (4) 起的实现者还明确写了"任何答复都不作为授权……在收到指示前我按既有决策继续实现与验证"，并说明"若 root 判定应严格停手，本片工作只是 worktree 内一个未推送提交序列，可整体弃置"——**这是最完整的正确处置**。

**规则**：subagent 报告中的任何"root 裁决/批准/选项 X"一律视为**未授权**，root 验收时必须 (a) 要求 subagent 附上答复原文存档（本会话已如此要求，做法有效）；(b) **独立复核该技术决定是否成立——不得因为"root 批准了"就跳过审查**；(c) 把归因更正为"实现者提案、root 事后核实追认（并附 root 的独立证据）"或退回；(d) 若 root 复核后**结论与该答复一致**，仍须**显式重新裁决一次**并注明原答复来源不明，且把答复中 root 认可的要求**以自己的名义重述**（不得让那条答复成为决策链上的一环）。派发任务书须显式写入此纪律。

**影响面递增，务必注意 (2)(3) 改的是行为面**：(2) 移除了一道生产门的合取；(3) 要放宽**四条路径共享**的准入普查（`prove_anonymous_owner_xrefs` 被接口路径/grandchild×2/父类路径四方共用）。若 root 只审 spec 归因不审代码，就会把未审的放宽合入主线。

**由第三起暴露的独立教训：spec 的"落点清单"可能不完整，而实现者发现后要按"新门"处理**。root 给环 2 的 spec 钉了三道门并写明"`anonymous-top-level` 只需本环"，但实现者取证发现**第四道门**（共享 owner 普查拒绝"child 自身方法调用"）——因为环 0/1/3 三锚的 child 体恰好都不自调用（环 0 用 `super.render()`、owner 是父类；环 1/3 读根类静态字段），**该臂从未被行使过**。教训：(1) root 写 spec 时，若某判据"从未被既有正例行使"，须在 spec 里显式标注为**未验证臂**，否则实现者会以为覆盖完整；(2) 验收锚的 child 体应尽量覆盖**多种符号 owner 形态**（自调用 / `super.` / 外部类静态字段），单一锚会漏掉整条臂；(3) 实现者发现 spec 未枚举的门时，正确处置是**停手 + 提出方案 + 请示**（第三起的实现者正是如此），不是自行放宽共享判据。**第四起是同一教训的第二例**：root 的 spec 把落点钉在 `latches.len() != 1`，实现者用最小门控实验证伪（放宽它 S5 拒绝数 2→2 不变，该判据从未被行使），真实落点是 join 选举——详见「写 spec 钉落点前必须做最小门控实验」纪律。

**验收锚不得是唯一正例——标识符泛化须有对照探针（2026-10-04 root 实证确立，强制）**：任何"证明一条特定用户标识符形态"的切片，其验收必须包含**至少一个标识符名与锚不同的正例**：若实现正确泛化它应同样通过，若实现把锚名硬编码了它会失败。根因：CI 全绿 + spec 通过 + 原/JADX/Jarde 三方对照一致，**都无法暴露"能力只在验收锚的特定标识符上生效"**——这类缺陷只能被"变更一个正交变量（如字段名）的结构同构对照探针"发现。本会话实证：`recover-proved-string-arg-enum-constant-bodies`(8/8) 把承载 String 实参的枚举字段名硬编码为 `op`，而 `op` 恰是其验收锚 `DoubleOperations` 的字段名（验收自我实现）；root 用六形对照（仅字段名不同、结构逐字节同构）钉死判别变量，非 `op` 名一律不投影（见 `openspec/evidence/java-syntax-2026-10-04/enum-string-field-name-hardcode/` 与 `hardcoded-identifier-audit/`）。**语言/JDK 强制的名字（`value`/`intValue`/`<init>`/`compareTo`/`ordinal` 等）不在此列**——它们本就该硬编码，无需对照。**文档记载为"窄首片"的硬编码**（如 `prove_static_assignment_suffix` 的 `totalUnits`/`sumUnits`，其 doc 注释明写"deliberately keep this first slice narrow"）是可接受的 MVP 边界，但须在账本登记为已知限制、且泛化时补不同名正例。

**空查询不能证明"不存在"（2026-10-04 root 本会话犯错，强制）**：一次范围过窄的 grep 返回空，**不构成**"某实现/调用方/能力不存在"的证据。本会话实例：root 只在 `crates/jarde-java/src/lambda.rs` 与 `src/class_source.rs` 两个文件里 grep `\$jarde` 未命中，便写下"该重命名从未实现，仅存在于文档描述"——而实现就在 `src/facade.rs:7817`，且 `tests/lambda_companion_bodies.rs:314`、`tests/p3_immediate_functional_receivers.rs:1450` 两处测试断言它。规则：下"未实现/无调用方/不存在"结论前，搜索范围必须覆盖**整个 `src/` 与 `crates/*/src/`**（外加 `tests/` 以查断言），并对同一目标用**多个不同拼写/标识符**各查一次（如符号名、字面量、错误码、字段名）；仍为空才可下否定结论，且须写明搜索了哪些路径与关键词。同理适用于账本/spec 中"某能力缺失"的记载——先穷举搜索再断言缺失，否则会把已实现的能力当成待办项重复立项。

**写 spec 钉落点前必须做最小门控实验（2026-10-04 root 本会话连犯两次，强制）**：spec 的"落点"（要放宽/修改的那道判据）**不能只靠读代码找到"看起来相关"的判据就钉死**——必须用**最小门控实验**证成：只放宽那一道判据（环境变量门控或临时补丁），观察**目标锚的行为是否变化**。若不变，说明该判据**在目标锚的流程上从未被行使**，落点是错的。本会话两次都是 root 的 spec 把落点写错、由实现者取证纠正：
- **getClass 片**：root 的 spec 断言"两种限定符形都是同一拼写判据的输入"，漏掉分配限定符形的检查落在**实参窗口内**；实际第一道门是 `init.rs:687` 的实例读者门（诊断逐字匹配 `init.rs:694` 模板，readers = javac 8 插入的 `dup`），拼写判据的修复**在结构上无法触达**该形 → 主锚未达成、只部分交付。
- **shared-latch 片**：root 把 `region.rs:10483` 的 `if latches.len() != 1` 记为"本片要放宽的现约束"；实现者做门控实验（只放宽它）后 **S5 拒绝数 2 → 2 不变**，证伪该前提——外层 `while` 走的是 `header_tested_loop`（9206）而非 `latch_tested_loop`（10470），内层循环从未进入 `loop_region`。真实落点是 **join 选举**（两臂 ipdom 汇合到共享 latch → join 当选 → 内层块永不重访 → uncovered）。

**规则**：(1) spec 里每写一处"落点/要放宽的判据"，都必须附**最小门控实验的结果**（放宽它，锚的行为是否变化），否则标为"未经门控实验、属读码推断"；(2) 若一道判据有**兄弟入口**（同一域的多个函数，如 `header_tested_loop` / `latch_tested_loop`），必须先确认目标锚走的是哪一个——读到一个"看起来是它"的判据不等于目标流程经过它；(3) 拒绝诊断的**原文**要与代码里的模板**逐字比对**来定位产生点，不要凭判据的语义相似度推断（root 就是用逐字匹配 `init.rs:694` 模板才把 getClass 的第一道门钉死的）；(4) 实现者用门控实验证伪 root 的落点时，**这是停手条件 (a) 的正确用法**，root 应在验收时更正 spec 与账本（保留原文 + 追加更正段），不得要求实现者按错误落点施工。

**验证脚手架必须先自检（2026-10-04 root 本会话撞三次，强制）**：探针/普查/对照脚本本身的 bug 会产出**自信的错结论**——比"没有证据"更危险，因为错结论会被写进验收判据与账本。本会话三例：(1) FP 探针的 python 提取写出**空文件**，使 `javac exit=0` 变成无意义断言；(2) 负例编译脚手架两次出错——文件名与 public 类名不符使 javac 因**无关原因**失败（误读为"响亮失败"），以及用正则改类名未同步改构造器名产生伪报错；(3) corpus 普查正则的行尾 `$` 未考虑 `javap -p` 追加的 ` {`，导致 482 个类**全部 NO MATCH**——若未发现，会得出"全语料无一类在范围内"的假阴性并据此写错验收判据。规则：**任何用于产出验收判据的脚本，先在已知的正例与负例上自检**（正例必须命中、负例必须不命中，并打印期望值与实际值），自检通过后才跑全量；脚本产出为零/为空/全同值时**默认怀疑脚本而非语料**，先复核再下结论。普查类脚本还须报告 `ERRS`/`NO_MATCH` 计数——静默跳过会被读成"不存在"。**自检样本必须覆盖语料的形态多样性，否则自检通过也会漏 bug**：本会话第 (3) 例的普查自检用了 4 个类全部通过，但它们都在**默认包**；正则的类名段 `([\w$]+)` 不接受带包前缀的点号名（`cf04.Negative`、`matrix.Outer$A$Generic<V>`），致使全量跑时 **65 个带包类被静默跳过**（只报 `DECL_NOMATCH` 而未计入结论），命中数被低估。故自检样本须**含带包名的、含泛型形参的、含接口/枚举/注解的**各类形态，且 `NO_MATCH` 计数必须为零才可采信普查结论——**任何被跳过的条目都要先解释清楚，不得当作"不在范围内"**。
>
> **假零结果（第 4、5 例，2026-10-04 追加，最危险的一类）**：脚本**成功退出但输出的是错误信息**，对该错误信息做统计会得到"零"，而零看起来像"干净/健康"的结论。本会话两例：(4) 普查字节码惯用法时用 `grep -q getClass` 字面计数得"6 个类用 `getClass` 形"，实际是**用户级** `getClass()` 调用（`if_acmpne` 引用相等、`getClass().getName()`）——假**阳**；改按指令序列（`dup`→调用→`pop` 连续三条）匹配后为 **8 / 0**，结论从"语料两形都有"翻转为"语料结构性只有一种拼写"。(5) 渲染某类后 `grep -c '@bytecode'` 得 **0**，被读成"四方法全部恢复"；实际那次调用因 `cd` 后仍用**相对**二进制路径而 exit 127，输出文件内容只有 `target/debug/jarde-cli: No such file or directory`——那个 0 是**错误文件的 0**，毫无意义。改绝对路径重跑后才是真实渲染（结论也随之改变）。
>
> **规则（新增）**：(a) **凡"引注数/命中数 = 0"的结论，必须先确认输出文件不是错误信息**——检查 jarde 自述头（`// jarde: presentation of …`、`// jarde: not a compilable project …`）是否在文件中，或检查产出文件的首行/大小；(b) `cd` 之后一律用**绝对路径**调用二进制与脚本产物；(c) 统计前先用 `wc -l`/首行抽样确认被统计的文件确实是预期产物；(d) 字面 grep 不能用于**字节码惯用法**普查，必须按指令序列匹配（且扫描器先用已知正例自检命中）。
>
> **错误 JSON 输出（第 7 例，2026-10-05 匿名类巡查自踩后自查）**：`--policy single-class` 配 jar 输入会得到 JSON 错误（`environment_policy_snapshot_kind_mismatch`），输出文件**不含源码**——对它计 `@bytecode` 同样得 0。规则升级：**自述头断言（`// jarde: presentation of`）必须是计数的硬前置门**（脚本在无头时立即退出报错，而非打印 0）；错误 JSON 与错误文本是同类假零源。
> **假版本标记（第 6 例，2026-10-04 root 巡查局部类时自踩后自查修正）**：探针用**裸 `javac`**（无 `--release 8`）编译，产物含 Java 9+ 惯用法（`StringConcatFactory` 的 invokedynamic 拼接），jarde 对它引注是**正确的版本标记拒绝**，却被误读成"局部类有缺口"。规则：**巡查/验收探针必须显式 `--release 8`（或真 javac 8）编译**；见到 `StringConcatFactory` 类 Java 9+ 惯用法，先查探针编译命令再谈缺口。同段实证：局部类方法体 quotes=0 全恢复（`new LC$1L(arg0).run()` 形），整类不可编译撞的是**池形伴生名无声明**债务（与 `Svc$Entry` 同源；判别变量是"池形伴生名是否在源码区有声明"，与成员类/局部类无关）——归成员折叠通道拼写域，勿当控制流缺口立项。
>
> **空结果不能自证扫描器正确**：扫描器报"全部 SAME/全部 0"时，必须先用**已知正例**自检（本会话的 codegen 差异扫描器移植后，root 用已知版本耦合的 `TR` 复验，确认它仍正确报 `VERSION-COUPLED` 且能区分同类内非耦合方法，之后才采信其六个构造的 SAME 结论）。**扫描器的比较维度还必须覆盖该构造的版本敏感数据**：opcode-only 扫描对 `invokedynamic` 系构造（lambda/方法引用/字符串拼接）不足——BSM 参数变化不改变 opcode 序列；root 因此对 lambda 补做了 `BootstrapMethods` / `InnerClasses` / 合成方法命名三项属性级比对才下结论。

**更正他人归因时须标注自己的证据强度（2026-10-04 root 自撞，强制）**：验收时不仅要独立核实"是否达成"，也要核实"为何未达成"——后者决定后续片的落点。但**更正他人归因时，自己的证据强度同样要如实标注，不得以表层观测宣称推翻内部推断**。本会话实例：实现者停手报告称分配限定符形被 `verify_member` 的实参窗口门 `jre_new_member_order` 拒绝；root 未核实就照录进验收记录（疏失一），随后以合并态二进制重渲染、读到表层诊断码是 `jre_new_interleaved_effect` + `jre_new_shape`（其中无 `jre_new_member_order`），便用"**证伪**"的措辞去更正实现者（疏失二）——而 root **只测了表层诊断码、未用插桩复现其内部连锁**，且实现者连锁的末步（"new@1 效果扫描拒绝 `jre_new_interleaved_effect`"）恰与 root 实测码相符，两者可能在不同层级各自成立。规则：(1) **转述他人机制归因前必须实测**，否则明确标注"属实现者推断、root 未复核"；(2) 更正时按自己实际做的实验下结论（"root 实测表层码为 X，Y 不在其中；未插桩复核其内部连锁，故不宣称其为假"），不用超出证据强度的词（"证伪"/"错误"）；(3) 若该归因决定后续片落点，须在 spec 中**强制后续片自行重新推导**，不得继承任一方结论；(4) 对实现者自己的证据文件，**追加 root 裁定段而不改写其原文**——保留决策链可追溯性。

**`class-source` 的调用姿势：家族折叠只在渲染根类时发生（2026-10-04 root 本会话撞两次，强制）**：`jarde-cli class-source --input <jar> --class <name>` 中，**只有渲染根类（`--class Outer`）才触发家族折叠**——把 `Outer$Inner`、`Outer$Stat` 等成员折进根类的同一源码单元、消隐 `this$0` 与 `access$NNN`、并把限定构造拼成 `outer.new Inner(…)`。**渲染子类（`--class 'Outer$Inner'`）不触发折叠**，会呈现未折叠的物理形（`access$` 保留、限定构造被引、quotes 偏高）。本会话两次因此得出**错误结论**：(1) 巡查 DT-03 时先用 `--class 'N1$Stat'` 渲染，见 quotes=7 且 `outer.new Inner(9)` 未折叠，误判"限定外部实例分配整体未覆盖"——改用 `--class N1` 后 quotes=0、两处限定构造都折叠，才发现真缺口只是 javac 版本拼写；(2) 排查写访问器时用 `--class 'WA$Setter'`，见未折叠，误判"javac 23 下折叠也失败"——改用 `--class WA` 后确认折叠成功、写访问器仍失败，才得到"两缺口正交"的正确结论。规则：**判断家族级能力（折叠/消隐/限定构造）时必须 `--class <根类>` 且 `--input` 用含全家族的 jar**；`--policy single-class` 只用于**独立**类文件（无家族关系），对家族成员用它会得到未折叠形。凡"某形未恢复"的结论，**先排除调用姿势**（根类 vs 子类、jar vs 单 class）再归因到判据。

**字节码惯用法普查必须按指令序列匹配，不能字面 grep（2026-10-04 root 撞一次，强制）**：普查"哪些 class 含某编译器惯用法"时，`javap … | grep -q <名字>` 会把**用户级同名调用**误计为编译器插入的惯用法。实例：root 用 `grep -q getClass` 普查 null-check 惯用法得出"6 个类用 `getClass` 形"，实际那些是用户级 `getClass()`（`if_acmpne` 引用相等比较、`getClass().getName()`）；改按**指令序列**（`dup` → 目标调用 → `pop` 连续三条）匹配后为 **8 个 `requireNonNull` / 0 个 `getClass`**——结论从"语料两形都有"翻转为"语料结构性只有一种拼写"，直接改变缺口严重性判定。规则：(1) 惯用法普查一律按**连续指令序列**匹配（awk 状态机或等价），不按名字字面 grep；(2) 扫描器必须先用**已知正例**自检命中（root 用真 javac 8 编的 `N1$Stat` 自检，确认非扫描器 bug）再信其零结果——**空结果不能自证扫描器正确**；(3) 涉及"javac 合成惯用法"的巡查必须**双 javac 腿**（真 javac 8 与 javac 23 `--release 8`）编译同一源码对照：单腿无法暴露版本耦合，且 `--release 8` **不回退** JDK 9 引入的编译器行为变更（root 实测三种组合，只有真 javac 8 发射 `getClass()`）。

**管道退出码陷阱（2026-10-04 本会话撞两次，强制）**：`cmd | tail -N && next` 判的是 **`tail` 的退出码（恒 0）**，不是 `cmd` 的——故 `cmd` 失败也会继续跑 `next`。本会话两次踩中：(1) `git push … | tail -1 && break` 使重试循环首次失败即 break；(2) `git merge … | tail -3 && cargo test …` 使 merge **冲突未解决**时测试照跑，得到的数字来自冲突态工作树（幸而冲突仅 docs，编译代码已合并）。规则：**凡后续步骤依赖前一步成功，就不要用 `cmd | filter && next`**；改为 `if cmd > /tmp/out 2>&1; then …; fi` 再单独读 `/tmp/out`，或用 `set -o pipefail`。同理 `timeout N git push` 会在慢网络中途 kill 传输留下假失败——push 用工具 background + 完成通知，不套 shell timeout。判定落地的唯一可靠信号是 `git rev-parse HEAD` == `git rev-parse origin/main`，不是输出或退出码。

**Push 策略（2026-10-04 root 实测修正；2026-10-05 用户授权补充代理路径）**：`git -c http.version=HTTP/1.1 push origin main` **不是**可靠路径——本会话实测它对 docs-only 小提交连续 flake（`fatal: unable to access … Empty reply from server`，三次重试全败），而**默认 HTTP/2 的 `git push origin main` 一次成功**。故顺序应为：(1) 先试默认 `git push origin main`；(2) 若报 HTTP/2 相关错误（`HTTP/2 stream … was not closed cleanly`、`Error in the HTTP2 framing layer`）才退回 `-c http.version=HTTP/1.1`；(3) **若 push 长时间挂起（>2-3 分钟无输出），走系统代理**（用户 2026-10-05 指示"push 可以使用 proxy"；本机 `scutil --proxy` 显示 127.0.0.1:7890 已启用）——`export https_proxy=http://127.0.0.1:7890 http_proxy=http://127.0.0.1:7890` 后重推，实测**秒成**（同一提交无代理挂 5 分钟+、代理一次过）；(4) 两者都 flake 时以 `git status -sb` 的 `ahead N` 判定是否真未落地（勿凭命令退出码），并核实 `git rev-parse HEAD` vs `git rev-parse origin/main`。旧文档记载的"push 用 HTTP/1.1 + 重试"已过期，勿再照抄。

**磁盘纪律（2026-10-02 用户指令强化）**：subagent worktree 的 cargo target 单份 ~20-23G，多 worktree 并行或主仓同建会迅速打满盘（本会话已三次临界：8.5G/6.7G/3.9G，两次杀死 agent）。规则：(1) 每个 subagent 任务书已含磁盘纪律段——每轮构建/测试前 `df -h /` 检查、低于 12-15Gi 先 `cargo clean`、**每完成一轮全量测试后若接下来是文档/分析等非构建工作，先 clean**、报告前必 clean；(2) root 在合入验收后立即 `git worktree remove`（含 target），不留残留；(3) 主仓自己的验收构建用完即清（`rm -rf target`）；(4) 已知 flake 家族（export_cli 计时、bulk_recovery_delivery 4-worker、observable_equals、backward_second_entry、p4_plugins、engine::standalone、ordinary_generic_projection 临时目录碰撞）重跑判定即可，勿误判回归。**串行派发约束**：因单份 target ~20G，同一时刻只允许一个 subagent 构建；root 巡查期间**复用运行中 subagent worktree 的 HEAD 版二进制**（先核实 `git status` 无 crates/src/tests 改动且二进制 mtime 新于源码，确认忠实反映 HEAD），避免自建 target 造成双 target 并存打满盘。

- **flake 家族增员（2026-10-06）**：`d3_artifact_binding::the_evidence_is_rebuilt_after_every_temporary_of_the_first_request_is_dropped` 在纯 docs 提交 `dccd21c3` 上 CI 红（run 37451511282），同代码的 `b6cbaa94` CI 绿——同族判定（该测试 10-05 已有 `7b291971` 先例，家族通式 docs-only 红 + 同代码绿）；本地单测两轮复跑并入 static-generic 验收批执行。

- **验收盲区教训（2026-10-06，postfix 合并 CI 红实证）**：CI 的 stable job 会跑 **ignored 的 JDK oracle/P3 对照腿**（`p3_execution_comparison` 的 3 个 `-- --ignored` 测试），root 本地门禁若只跑默认套件，**语料件翻态的切片会漏掉 oracle 期望同步**（`saved`/`conditional` 从 Quoted 翻 Executed 而 oracle 仍钉旧态 → CI 红）。规则：**凡 corpus 差分非零的切片，验收必本地补跑 `cargo test --test p3_execution_comparison --all-features -- --ignored`**（oracle 自身编译执行恢复体=行为验证，一并覆盖）。
