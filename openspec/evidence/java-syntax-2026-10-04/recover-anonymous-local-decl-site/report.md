# recover-anonymous-local-decl-site 实现与门禁报告

实现者：coder subagent（2026-10-04，worktree 基线 `537ff21b`）。范围：tasks 1.1–3.2（3.3 留 root）。所有数字实测于本 worktree 最终文件状态。

## 五处判据 + 第六道门的落点

| 判据 | 落点与实现方式 |
| --- | --- |
| 1（根方法门） | `project_class_source_anonymous_super` 的返回门改为形状条件化：`DirectReturn` 形保留 `descriptor == "()Lparent;"` 原检查逐字不动；`LocalDeclInitializer` 形无返回描述符检查（锚 `main` 为 `void`），父类源码名来自声明初始化值 `new` 操作数的 owner（`allocation_type`），与左端重拼同一来源。未"忽略返回类型"。 |
| 2（站点扫描） | `class_source_direct_return_new` **零字节未动**。新增共享扫描 `report.rs::class_source_anonymous_site`：直返候选优先（字节等价保留既有语义），否则对声明初始化位计数、恰 1 才产站点；目标名必须池形（含 `$`）。`class_source_anonymous_return_site` 携带 `AnonymousSiteShape` 返回四元组；AST 保留门改用共享扫描。"分配点之后语句整体 structured 无引注"由发射层兜底（`anonymous_decl_site_body_unproved`，见 3）。 |
| 3（左端重拼） | 在既有 `AnonymousOverride` 同一接缝上：emitter 新增 `anonymous_override_matches`（谓词与表达式路径一致），`StmtKind::Declare` 臂在"初始化值恰为匹配分配点 ∧ 左端拼写的正是匿名池名"时以父类源码名重拼。三项检查：(i) 既有 `anonymous_super_source_type_unproved` 门对两位形继续生效（`nested-super-parent` 负例）；(ii) 发射后幸存者检查——投影文本仍含匿名 child 池名即拒绝（`anonymous_decl_site_retype_incomplete`；池名幸存必然不可编译或被结构反射消费，拒绝先于发布）；(iii) 既有 owner 普查拒绝一切非分配/构造/捕获位的 child-owner 引用（锚的 `instance.render()` 字节码 owner 本就是 `Base`，`javap -c` 实证；`unresolvable-child-read` 负例证其响亮）。无 `var`/`Object` 兜底。 |
| 4（三道前置） | 原样遵守；root 2026-10-04 裁答追加更正注记于 design（第二道既有约束 `anonymous_child_methods_incomplete` 的发现与对齐处置），见 tasks 1.5。 |
| 5（共享扫描遏制） | 判别位 `jarde_java::report::AnonymousSiteShape` 由 `class_source_anonymous_site` 按语句位置产生，随站点元组（现五元组）进入 `_anonymous_return_sites`。接口路径前置加在 `project_class_source_anonymous_interface` 的**父类委派分支之后**（委派接受两位；接口自身路径要求 `DirectReturn`，否则 `anonymous_interface_site_shape_unsupported`）；嵌套接口路径 `proved_nested_anonymous_site` 只取 `DirectReturn`。 |
| 第六道门（1.5，root 裁决 A） | 父类路径 child 门的 `allocations.is_empty()` 合取去除，与接口路径（facade ~3670-3688，放宽前逐字核实仅 `scan.complete`）对齐；`scan.complete` + `complete_anonymous_method` 全量保留；grandchild 路径（~4400）合取不动。配套三类负例全部响亮拒绝（见下）。 |

## 主锚可观察行为对照（`anonymous-super-args`）

- 放宽前：`AnonymousSuperArgs$1 local2 = new AnonymousSuperArgs$1((java.lang.String) text("super-label", "explicit"), number("super-value", 17), local1);`（main 中部，后随两条 `println`）；root+Base 源集 `javac --release 8` **退出 1**（`找不到符号`）。
- 放宽后：`Base local2 = new Base((java.lang.String) text("super-label", "explicit"), number("super-value", 17)) { java.lang.String render() { AnonymousSuperArgs.event((java.lang.String) new java.lang.StringBuilder().append("body:").append(local1).toString()); return new java.lang.StringBuilder().append((java.lang.String) super.render()).append(":").append(local1).toString(); } };` —— 左端重拼 `Base`、捕获读取重拼 `local1`、后随语句原样；`javac` **退出 0**；`java -Xverify:all` 事件日志与原 class **逐行一致**（`arg:capture|arg:super-label|arg:super-value|base:explicit:17` / `explicit:17:captured`）。
- `-g` 对照腿：同形投影、左端类型来源一致（`Base instance = new Base(…) { … }`，`new` owner 而非 LVT；LVT 计数 5 vs 锚 0）；javac 0、日志逐行一致。

## 负例（全部响亮，呈现逐字节相同）

| 负例 | 形式 | 拒绝落点 |
| --- | --- | --- |
| 嵌套父类（`ParentCarrier$Holder`） | 源级冻结 | `anonymous_super_source_type_unproved` |
| 同方法两个声明初始化位 | 源级冻结 | 无站点（每方法计数 2），物理文本 |
| 初始化值非唯一分配点 | `two-decl-sites` CP 补丁（`$2`→`$1`，verifier-valid） | 无站点（单分配证明关闭），物理文本，两处 verbatim |
| 后续读取在父类上不可解析（匿名体自调用 `extra()`） | 源级冻结 | `anonymous_interface_child_additional_use`（owner 普查） |
| 不可拼写 owner 的分配（child 体 `new DeepCarrier.Mid.Leaf()`） | 源级冻结 | `anonymous_child_methods_incomplete`（child 门，恢复层引用分配） |
| 嵌套匿名分配（child 体 `new …$1$1`） | 源级冻结 | `anonymous_interface_child_additional_use`（grandchild 的 EnclosingMethod 构成 child-owner 引用） |
| 自引用分配（锚 child 的 StringBuilder 类常量改指自身；jarde-readable、刻意非 JVM-verifiable，无运行腿） | 补丁派生 | `anonymous_interface_child_additional_use`（child 体持有自身 owner 引用；实测落点为拒绝，站点唯一性不变量不受影响） |

遏制负例（接口分配点在声明初始化位）：呈现**逐字节相同**（SHA-256 `1badfcb5…` 两腿一致）；state `absent`→`refused`（诊断级，reason `anonymous_interface_site_shape_unsupported`）。

## corpus 双腿扫描（同一 fixture 集，84 渲染/腿）

- 差异恰 **2 处**渲染：`anonymous-super-args::AnonymousSuperArgs`（本片锚）与 `anonymous-super-args-debuginfo::AnonymousSuperArgs`（同形 `-g` 腿）——均为赋值初始化形，无第 2 个差异类。
- **接口匿名形零差异**（含 `anonymous-interface-basic`、`anonymous-inner-this`、`anonymous-double-site` 等全部接口/内 this fixture）。
- SUMMARY 差异仅为 4 个新负例的请求退出码（0→4，渲染文本逐字节相同）。
- 过程记录：首轮扫描曾抓到一处真实回归——`anonymous-inner-this` 因普通可命名类声明初始化（`Inner inner = new Inner();`）被计为站点而未投影；修复为声明初始化位候选仅接受池形（`$`）目标后逐字节恢复。该教训已写进扫描注释与 tasks 2.1。

## 门禁数字（最终文件状态）

- `cargo test --workspace --tests --locked --no-fail-fast`：**296 目标 / 2943 passed / 0 failed**（主线基线 2937 + 本片新增 6：锚投影、`-g` 腿、遏制逐字节、负例五合一、双站点+非唯一、自引用派生）。jarde-reader 的 fixture 总量计数测试按新增 26 类重测（508/2310/251/1689/8，注释追加缘由）。无 flake 需复跑。
- `cargo fmt --all -- --check`：通过。
- clippy：`sed -n '46,76p' .github/workflows/ci.yml` 逐字生成（`--all-features`、29 项 `-A`、`-D warnings`）：**退出 0**。
- `openspec validate --all --strict`：**270/270 通过**（worktree 内实跑；本片仅改自身 change 的 design/tasks 内容，未新增/改名 spec 文件）。
- corpus 双腿扫描：见上。
- `git diff --check`：干净。
- 磁盘纪律：每轮构建前 `df -h /`（最低 24Gi，未触发 12Gi 清理线）；报告前 `cargo clean`。

## 遗留缺口（如实登记）

- 接口路径遏制负例的 `state` 字段 `absent`→`refused` 属诊断级变化（渲染与行为零变化）——与上一环 IP2 的同类登记一致，corpus 扫描的"报告 JSON 投影状态字段比对"仍是已登记的独立债务。
- 判据 3 的健全性论证边界：重拼将声明类型收窄为 `Base`；若原源码声明的其实是更宽的父类型（无 LVT 时不可恢复）且后续存在重载选择依赖该宽类型（如 `show(Object)`/`show(Base)` 重载），重编译的重载选择可能不同——design 判据 1/3 钉死"重拼为父类源码名"的选择与其"至多更精确、不新增不可解析调用"的论证口径，本片按此实现并如实登记该理论边界。
- 根方法带参形、根方法返回父类子类型/接口形：仍属未立项切片（Non-Goals 维持拒绝）。
- 自引用分配负例的补丁输入非 JVM-verifiable（构造器描述符在复制后不解析），故无 `-Xverify:all` 运行腿；其测量目标是 jarde 的路由（拒绝），已如实记录。

## root 裁答

两次 `ask_parent` 答复的逐字原文存档于 `root-replies-verbatim.md`（fixture 冻结形式/负例归属/`-g` 腿/openspec 校验四问；第六道门的选项 A 裁决与三类负例前提）。本报告与 tasks.md 中的一切"root 裁决"均以该存档为准。
