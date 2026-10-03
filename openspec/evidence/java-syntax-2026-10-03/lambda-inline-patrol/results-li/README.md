# Lambda 伴生体内联/重命名（recover-lambda-inline-bodies）实现取证（2026-10-03）

实现侧取证与验收记录（`-li` 后缀目录），对应 OpenSpec change `recover-lambda-inline-bodies`。
巡查侧背景见[上级 README](../README.md)；行为基线 orig.out 不变（`hi!`/`45`/`[b, aa]`/`8`）。

## 装配关系取证（任务 1.1 结论）

- **呈现位点**：lambda 表达式由 `lambda@1` 规则在 build 层装配为
  `ExprKind::Lambda { params, body: Call { receiver: Path(Owner)|Local("this"), name: "lambda$…", args } }`
  （`crates/jarde-java/src/build.rs` 的 `lambda_expr`）。伴生以**普通 private static（或 private 实例）合成成员**
  进入类文本：类源装配（`src/facade.rs` 的 class-source 成员环）对每个成员各跑一次恢复，
  `lambda$…` 成员与普通方法同路径拼出声明与体（`src/class_source.rs` 的 `block_member`）。
- **证明通道**：invokedynamic → BootstrapMethods 行 → LambdaMetafactory.metafactory 静态句柄 +
  实现句柄（`crates/jarde-java/src/lambda.rs`，A04 检查）。本片**未改**该通道，仅加宽其后的
  伴生候选收集（`synthetic_lambda_helper_candidates`：去掉 int-only 限制，任意描述符）。
- **内联判据数据面**：伴生体的判据全部取自同运行保留的成员 AST（`ClassSourceMethodAst`：
  `program`/`parameter_names`/`complete_code`/`instruction_bcis`）与调用方保留投影
  （`LambdaHelperProjectionSource`：program/facts/declaration/member）。类级单用途证明复用既有
  `lambda_helper_census_refusal` 全类普查（直接调用、ldc 句柄、全部 bootstrap 行、CONSTANT_Dynamic
  可达闭包都必须恰为候选位集）。
- **冲突基线重放**：主线转写与巡查冻结的 `results/Y1.java` 逐字一致；`javac --release 8` 对其
  （补 `Y1$StrFn` 接口后）编译复现整类失败：`符号lambda$viaLambda$0(String)与Y1中的
  compiler-synthesized 符号冲突`（[y1-before-javac.txt](y1-before-javac.txt)）。

## 落地实现

- `crates/jarde-java/src/lambda.rs`：伴生候选收集泛化（任意描述符/参数/返回；保留
  LambdaMetafactory、同类、`lambda$` 标记、kind 6/7 过滤）。
- `crates/jarde-java/src/report.rs`：新通道 `plan_class_source_lambda_inline`（单 return 体判据、
  参数按位绑定、捕获避免、调用方作用域不遮蔽）与 `emit_class_source_lambda_member`
  （成员级一次性重发射，含未修改基线发射供守卫比对）；替代原 primitive-int 专用
  `emit_class_source_lambda_helper`/`rewrite_lambda_arithmetic`（int 家族为其真子集）。
- `src/facade.rs`：类源装配的伴生通道重写——按伴生分组、census、单用途判定、内联/重命名决策、
  成员级文本编排（重命名可叠加在被编辑成员上）、全额计费后原子提交；多用途（≥2 位点）保持
  物理呈现并登记诊断。
- `src/class_source.rs`：`matches_current_text`（块内区域守卫）、`renamed_declaration`
  （名字恰现一次才改）、`lambda_rename_text`（重命名拼接，含叠加基座）、`lambda_projection_text`。

## 判据（root 决策 1/2/3 的实现形态）

- **内联**：伴生=单 `return` 表达式、锚点覆盖全部物理指令、无异常表、体不含
  嵌套 lambda/条件/短路/局部写、全部 Local ∈ 伴生形参（kind 7 允许 `this` 透传）；
  形参↔调用实参按位绑定（尾实参须为位点参数的cast链）；两实参结构相同拒绝；非纯捕获被读>1 次拒绝。
  lambda 参数名沿用伴生形参名；若与**调用方任何作用域名**（形参、声明局部、自由读）冲突——
  javac 禁止 lambda 参数遮蔽外层局部——回退用位点参数名（free_name 保证不冲突）；
  两种命名都冲突则走重命名分支。
- **重命名保守**：`lambda$name$N` → `lambda$name$N$jarde`（类内无同名成员、声明中名字恰现一次）；
  声明行与调用位同步重写；`// @method` 证据注释保留物理名，成员内加说明性 marker。
- **边界**：多语句直线体与分支体一律重命名（不内联）；`X::m` 方法引用语法不做（呈现不动）；
  多用途伴生不内联、不隐藏、不重命名（保持现呈现并登记）；无 `lambda$` 伴生的类不进通道。

## 验收记录

- 变体/负例转写前后：`v1-before/after`、`v2-before/after`、`m1-before/after`（`results-li/` 下）。
- Y1 命中输出：`y1-after.txt`（四位点内联 + 五伴生省略）；重编 `--release 8` 通过、
  `java -Xverify:all` 输出与 orig.out 逐字一致（SHA 见 `sha256.txt`）。
- 三方对照：`three-way/`（原 class / 固定 JADX / Jarde 重编，逐路径输出一致）。
- corpus 双腿：`corpus-scan.txt`（HEAD 基线 vs 本片；差异仅 lambda 家族形态；无 lambda 类逐字一致）。
- 门禁：见根报告（fmt/clippy(CI `-A` 清单)/全仓测试/openspec validate）。

## 登记的遗留

- `X::m` 方法引用语法呈现（行为已对，独立后续片）。
- 多用途伴生类仍不可编（保持忠实优先；见 root 决策 3）。
- 直线**多语句**体（如 `int t = v+1; return t*2;`）走重命名分支（块体 lambda 需 AST/emitter
  扩展，非本片）；类仍可编、行为一致。
