# 冻结行为 fixture 守卫覆盖：取证、分类与负向自检（2026-10-07）

本目录是本片（`recover-fixture-behavior-guard-coverage`）的证据：6 个在范围内 fixture 的 class
清单与 SHA-256（1.1）、**实测**行为 golden 与 javap 关键事实（1.2）、逐 fixture 分类（1.3）、
2.3 负向自检的扰动与失败断言，以及 3.1 门禁原文。

工具链：`java`/`javac` = OpenJDK 23.0.1（javac 23.0.1）；全部 fixture class 的 `major version = 52`
（Java 8）。命令一律绝对路径，`-cp` 指向 fixture 目录（嵌套布局用 `v8/`）。

## 0. 开工复核：当前 CI 引用（proposal 的范围更新已过期一半）

proposal 的范围更新基于 2026-10-04 的实测（环 0/环 1 合入后）。本片开工（2026-10-07，HEAD
`fdef2537`）时**环 2、环 3 也已合入**，故先复核当前引用再动手：

| fixture | HEAD 上的 CI 引用 | 本片处置 |
| --- | --- | --- |
| `anonymous-member-base` | 无 | 补呈现腿 + 行为腿（本片） |
| `anonymous-top-level` | **`tests/anonymous_supertype_return.rs`（环 2，合并 `23b69bcf`）** | 补行为 golden + 子类字节序锚；**不重复**其根投影断言 |
| `lambda-body-inline` | 无 | 补呈现腿 + 行为腿（本片） |
| `enum-arity` | 无 | 补呈现腿 + 行为腿（本片） |
| `package-info-basic` | 无 | 补呈现腿 + 行为腿（本片） |
| `short-circuit-left-false` | 无 | 补呈现腿 + 行为腿（本片） |
| `anonymous-super-args` | `class_source.rs`、`ctor_reorder_dispatch_guard.rs` | 不重复（proposal 已排除） |
| `anonymous-super-dispatch` | `anonymous_parameterized_root.rs`、`ctor_reorder_dispatch_guard.rs` | 不重复（proposal 已排除） |

**由此产生的一处措辞更正**：proposal/tasks 2.1 说 `anonymous-top-level`"当前物理文本呈现、
后续切片解锁时主动更新"——该切片（环 2）**已经落地**，故它的当前呈现是**投影形**
（`return new Base(choose()) { … }`），本片按投影形钉锚并在测试文件里注明环 2 已解锁；若照原措辞
钉物理文本，会把环 2 的已交付成果钉成回归。

## 1.1 class 清单与 SHA-256（实测）

```text
fe436b9b44e420fa9b9c768661b3405389652cb56136ea82eb2e83728bbcd31b  anonymous-member-base/AnonymousMemberBase.class
cdc10bfb3d1396779edbabd3e2d258c32996a915e00d33c1af0ff8445af38b01  anonymous-member-base/AnonymousMemberBase$1.class
4105fff279d1f89f4d50676bc704430b9ec48ae54fe6ed64e3d107a52ea49f31  anonymous-member-base/AnonymousMemberBase$2.class
19ab42816fc39f8ab685170c0d3ff833741fd6e2459b6d0503be43c056c882bb  anonymous-member-base/AnonymousMemberBase$Outer.class
ab1f19e36382f8a96519b0450e3327a9e41d2aa1fdb2262ed3310f92febbb81c  anonymous-member-base/AnonymousMemberBase$Outer$Base.class
132207f90cbec6aab8ec7070628ffdfb2dbe5830494c314ae80ff8b70b121c43  anonymous-top-level/AnonymousTopLevel.class
89db5ebc180357b82dd2824637037a1b926f022209378b9cfdc48e38c1de6d28  anonymous-top-level/AnonymousTopLevel$1.class
d26aa60934ee434e5c486b635da16dd08307d1248aaae00da75a0321b1313460  anonymous-top-level/Base.class
05d77405e783870842f7b70fc38dae5f2ad67879690f7fcac7fc58ba32593c12  anonymous-top-level/Renderer.class
4293d266fa4b00a7f77fa2eeb64cb9cecaaafbd982bd351259ba584cad1eacce  lambda-body-inline/IntAction.class
e429024c15ac2a7f81523c1550a6c8687a37a0ea76dbbe32b935038136ef0c66  lambda-body-inline/LambdaBodyInline.class
f82eccdff0bed594b070e2e60d500c64cd4246016e4d553d947d32ba2e05d361  enum-arity/v8/probe/Empty.class
c20c298f799535727e9f52d19fad236637956efa7cacdecece397cf0a2be5433  enum-arity/v8/probe/EnumArityRunner.class
7607cef8157c41187595798efa2fbda9404f8c0b6280abbfe2e30aa96a84cb3a  enum-arity/v8/probe/Four.class
4f70b8c32bcdc2184093e5b67e7c4aad480bea0a74b26c694e1c374a586bab60  enum-arity/v8/probe/One.class
b424f71d3bd0794bf58a8e1674fe5d51995093ed35cbe435449c17e49817fbf1  package-info-basic/v8/p/Check.class
a43c23ab884e4caedd53afccf36272255e8a06f42d2642befa1bfe0f7162e240  package-info-basic/v8/p/package-info.class
a766f7a14a7ef71e255555314b88386c11be924ff27bdb3ec880cf09a607cb9d  short-circuit-left-false/ShortCircuitFalse.class
```

布局核对（proposal 的"布局陷阱"段）：`enum-arity` 的 class 在 `v8/probe/`（包 `probe`）、
`package-info-basic` 在 `v8/p/`（包 `p`），其余四个平铺。`v8/` 是生成标记而非包名——jar 条目名
是 `probe/…`/`p/…`，测试里逐个核对过（`include_bytes!` 用磁盘路径，条目名用包路径）。
`enum-arity`/`package-info-basic` 的 `SHA256SUMS` 与上表逐字一致；`anonymous-top-level` 的四个 SHA
与 `openspec/evidence/java-syntax-2026-09-25/anonymous-top-level/evidence.md` 的表格一致。

## 1.2 实测行为 golden（原始 class，`java -Xverify:all`）

6 个 fixture 的 README **没有一个记录过实际运行输出**，故全部实测；下表输出与各 fixture 既有证据
目录里记录的 `original-run*` 文件**逐字节相同**（脚本对比，比对时去掉 replay 脚本追加的 `exit=N` 行；
自检：任一实测文件为空即中止）：

| fixture | 运行类 | 实测输出（即 CI golden） | 与既有证据对照 |
| --- | --- | --- | --- |
| `anonymous-member-base` | `AnonymousMemberBase` | `normal=8;events=outer,argument,base(7),anonymous`⏎`null=nullOuter` | = `…/2026-09-25/anonymous-member-base/original-run.txt` |
| `anonymous-top-level` | `AnonymousTopLevel` | `value=23:captured`⏎`events=capture,choose,base(23),render`⏎`counts=1,1,1,1` | = `…/2026-09-25/anonymous-top-level/evidence.md` 记录块 |
| `lambda-body-inline` | `LambdaBodyInline` | 三行（`created …`/`first …`/`second …`） | = `…/2026-09-25/lambda-body-inline/original-run.txt` |
| `enum-arity` | `probe.EnumArityRunner` | `empty=0`⏎`one=ONLY:0/1`⏎`four=[NORTH, SOUTH, EAST, WEST]` | = `…/2026-09-27/enum-arity/original-run.log` |
| `package-info-basic` | `p.Check` | `true` | = `…/2026-09-27/package-info-basic/original-run.log` |
| `short-circuit-left-false` | `ShortCircuitFalse` | `false-result=false,calls=0`⏎`true-result=true,calls=1` | = `…/2026-09-25/short-circuit-left-false/original-run.txt` |

golden 逐字写在 `tests/fixture_behavior_guards.rs` 的 `*_GOLDEN` 常量里（含末尾换行）。

### javap 关键事实（实测）

- `anonymous-member-base`：`AnonymousMemberBase$Outer$Base.<init>` 的字节序是
  `putfield this$0`(BCI 2) → `invokespecial Object.<init>`(BCI 6) → `putfield value`(BCI 11) →
  `access$000("base(…)")`(BCI 38)；呈现把 `Object` 调用移到最前（`ctor_order` 只允许越过
  `java/lang/Object.<init>()V`）。`AnonymousMemberBase$1.<init>` 在 BCI 2–6 是
  `dup`+`invokestatic java/util/Objects.requireNonNull`+`pop`——javac 9+ 的限定接收者空值检查拼写
  （该拼写正是 `recover-javac8-getclass-null-check-idiom` 在飞的域），故该构造器本次未被恢复。
- `anonymous-top-level`：`AnonymousTopLevel$1.<init>` 字节序是 `putfield val$captured`(BCI 2) →
  `invokespecial Base.<init>`(BCI 7)；`Base(long)` 的构造器会调用 `AnonymousTopLevel.event`（用户
  代码），故合成组**不得**越过它——呈现保持字节序，其源码形因此故意不可编译（响亮的失败，而非
  静默改序）。
- `lambda-body-inline`：`LambdaBodyInline` 有合成 `lambda$build$0(II)I`（flags `0x100a`）与
  `build(I)LIntAction;` 的 invokedynamic 站点；`events` 字段的 `Signature`
  （`Ljava/util/List<Ljava/lang/String;>;`）未投影（呈现为裸 `java.util.List`，javac 因此报
  unchecked 提示）。
- `enum-arity`：`Empty`/`One`/`Four` 分别 0/1/4 个常量，三者均 `extends java.lang.Enum<…>`、私有
  构造器、`$values()`/`$VALUES`；类 `Signature` 未投影（呈现带 `class_generic_source_unproved` 注释）。
- `package-info-basic`：`package-info.class` major 52、flags `0x1600`
  （`ACC_INTERFACE|ACC_ABSTRACT|ACC_SYNTHETIC`）、0 字段 0 方法、1 个
  `RuntimeVisibleAnnotations`（`java.lang.Deprecated`）。
- `short-circuit-left-false`：`assign(Z)V` 的字节码是
  `iload_0; ifeq 14; invokestatic rhs; ifeq 14; iconst_1; goto 15; iconst_0; putstatic result`——
  一次写入，右侧在短路分支内。

## 1.3 逐 fixture 分类

| fixture | 渲染文本可编译？ | 分类 | 呈现腿（默认套件） | 行为腿（`#[ignore]`） |
| --- | --- | --- | --- | --- |
| `anonymous-member-base` | **否**（`jarde_refused_body();` ×2 + 根的两个未恢复方法） | 不可编译（钉事实） | 限定接收者拒绝 + 成员基类构造器顺序 | 原 class 运行 golden + javac 非 0 与记录诊断 |
| `anonymous-top-level` | **是**（根投影 + `Base`/`Renderer` 支持源） | 可运行且有基线 | 子类捕获写在 `super` 前（环 2 的根投影由 `tests/anonymous_supertype_return.rs` 钉） | 原 class 运行 golden + 重编运行逐行一致 |
| `lambda-body-inline` | **否**（`main` 未恢复 → `jarde_refused_body();`） | 不可编译（钉事实） | 转发箭头 + 改名 helper 语句序 + `main` 拒绝 | 原 class 运行 golden + javac 非 0 与记录诊断 |
| `enum-arity` | **是** | 可运行且有基线 | 0/1/4 常量头 + runner 的常量读取 | 原 class 运行 golden + 重编运行逐行一致 |
| `package-info-basic` | **是** | 可运行且有基线 | `@java.lang.Deprecated` 在 `package p;` 之前（整文本相等） | 原 class 运行 golden + 重编运行逐行一致 |
| `short-circuit-left-false` | **是** | 可运行且有基线 | 一次写入、右侧在 `&&` 内 | 原 class 运行 golden + 重编运行逐行一致 |

**当前呈现锚与将来的翻转**（避免后续切片被误判为回归）：`anonymous-member-base` 是
`present-proved-java-structure` 5.3 的 `this$0`+捕获+super 实参三者并存锚，`lambda-body-inline` 是
其 5.2 的多语句 lambda 体锚——两者当前都钉在"拒绝/转发"形上，解锁切片 MUST 在同一次提交里更新锚；
`anonymous-top-level` 的呈现已由环 2 解锁并钉为投影形。测试文件头部与各锚处都有该标注。

## 2.3 负向自检（扰动 → 新锚失败 → 完整回退）

- **选中的 fixture**：`anonymous-top-level`（本片真正钉了锚的一个；且扰动正是本片事故的机制）。
- **扰动**：`crates/jarde-java/src/ctor_order.rs` 的
  `call_cannot_run_user_code`（`present_prologue_first` 的越序门）临时放宽为"任何构造器调用"——
  即把 `target.owner() == "java/lang/Object" && name == "<init>" && descriptor == "()V"` 三条合取
  换成 `true`。这正是 `recover-synthetic-ctor-super-order` 出事故的那个洞（越过会跑用户代码的
  super 调用）。
- **结果**：默认套件 7 项中 **1 项失败**、其余 6 项仍通过（扰动只影响越序门）：
  `anonymous_top_level_pins_the_childs_capture_write_before_the_super_call` 失败，断言差为
  `left: ["super(seed);", "this.val$captured = arg3;", "return;"]` /
  `right: ["this.val$captured = arg3;", "super(seed);", "return;"]`——呈现被改成 `super` 在前。
  原文：`03-self-check-perturbation.out`（`test result: FAILED. 6 passed; 1 failed; 6 ignored`）。
- **`#[ignore]` 行为腿在同一扰动下仍通过**（`03-self-check-perturbation-ignored.out`，
  `test result: ok. 6 passed`）：本 fixture 的子类顺序在该运行里不可观测（`Base` 构造器不虚分派），
  这正是设计选择"呈现腿留在默认套件"的理由——改序类回归由呈现腿在每次 `cargo test` 上可见。
- **回退**：`git checkout -- crates/jarde-java/src/ctor_order.rs`；回退后
  `git diff --stat HEAD -- crates src` 为空、默认套件 7/7 通过、行为腿 6/6 通过。扰动**未留在代码里**。

## 3.1 门禁原文

见本目录 `06-gates.md`（实现者实跑：fmt、CI 46–76 逐字生成的 clippy、workspace 全目标测试、
`openspec validate --all --strict`、`git diff --check`、corpus fingerprint）。原始日志：
`04-gates-clippy.out`（46–76 逐字命令）、`04-gates-clippy-deny.out`（追加 ci.yml 第 76 行的
`-D warnings` 后的 CI 等价形）、`06-workspace-tests.out`。
