## Context

动机见 proposal。原始完整基线 `/private/tmp/jarde-testnestedloops-current-baseline-root-v1-20261018/execution.json` SHA256 `4f48e3b14738cb4d3746763290788f525bba4bb1a1a3d47aebcf0111e6f8d482` 已root实际独立接受，20命令40raw。child 两方法/无字段，目标45条物理指令/119B；原始双 profile/双 VM 共32case同原。8公开请求均Structured；当前泛型形参与两层foreach正确，缺口为头类型固定Object、正文String cast。JADX两份完整输出实际为String头。完整生成矩阵75命令150raw、20编译18成功、36运行288case同原、18wrong CodeSource拒绝亦已独立接受；两失败仅JADX outer的javac8框架major55边界，无runtime，见results。本片尚未实现。

源码边界：`build.rs::iterable_for_each_candidate` 已核 iterator 初始化/声明owner、phi/use、next→cast→store与handlers，最终固定创建 Object 头并保留原body绑定。`facade.rs::prove_generic_list_parameter_body` 在已有 generic-call 事务内证明直接 List formal/foreach 来源，`class_source.rs::ordinary_parameterized_declaration` 后置核 Signature/erasure/source spelling，最终闭包重验 staged proof。`report.rs` 保留 opaque 同次 AST/Program，已有原子投影和 emitter。原始方法 report/map 与投影后 method/class 文本是不同平面。

## Goals / Non-Goals

**Goals:** 把已经证明的两种能力在同一源码事务中结合；原目标声明、变量名字、赋值/声明语句、作用域和写入位置逐节点保留。两层头均应类型化，不只处理内层容易的 Declare 形。

**Non-Goals:** 不把 s1/s2 变成新的头绑定，不删除其声明或赋值；不清理冗余 iterator 声明、Object overload wrappers 或恢复原名黄金。后者仍是明确质量欠账，不能计作本片或CF11整体完成。unsupported List 类型形和一般变量作用域重构不进入本片。

## Decisions

1. **放在现有完整泛型事务，不放宽循环识别。** 直接formal物理slot、iterable读BCI、唯一foreach锚点与源码Signature E必须属于同一成员/类定义。先使用现有完整List参数与ordinary Signature erasure/name证明，再将精确可拼写的E交给opaque AST投影。不能从打印的 `List<String>` 文本或正文cast单独猜E；iterator-only的现有空actual-type集合不证明cast==E，必须显式合并这一约束。泛型证明拒绝/延迟时，保留旧表示。不要重新分析第二份Reader/SSA。
2. **保留body绑定，避免新增作用域机制。** 对受证 `ForEach(Object synthetic : formal)`，要求直接首句是 Declare(Some(Cast(Local synthetic))) 或 Assign(Cast(Local synthetic))，cast类型精确等于E；synthetic必须是该头既有合成绑定，其唯一消费就是首句cast，锚点与物理next/cast/store一致。改头ty为E，删掉首句RHS的cast包装，仅留下同一个typed synthetic读取；首句kind/name/statement origin及其它AST逐字保持。外层s1提前声明、零次迭代、循环后读取和内层s2作用域自然保留。没有必要移动局部声明或按同打印名字重绑物理局部。
3. **沿用物理/异常证书并核source shape。** 原始next/cast/store均在同次已受证foreach；物理opcode确认是真实checkcast而非仅descriptor呈现包装；精确canonical调用、单消费者和原handler集合不放宽。cast必须在直接首绑定、用户效果之前；拒绝try子作用域、前置效果、多消费者、错BCI/错类型和非直接formal。typed头的隐式cast仍先于原目标写入，之后保留的局部copy不能抛错，故错误元素/next失败不改变最后成功赋值。合并cast来源到头的既有OriginSet，store来源仍留在原绑定。
4. **复用opaque AST与既有发布路径。** 在已有AST投影/原始emission核对/emit/GenericCallProjectionState事务中组合typed投影与原call-cast/removal，避免独立正文改写通道。输入优先复用现有formal-slot/解析元素type元组和opaque AST，优先用既有AST/计划相等在最终generic闭包重验；若确有跨crate必要才添加最小doc-hidden记录，必须说明哪些既有事实不足，不能新增公有序列化模型或第二套全局证书。所有新遍历、clone和输出先计费/poll；Stop沿原转换路径传播，末尾一次发布。
5. **分清输出平面。** RecoveryReport.text/quality/map仍是同次raw恢复产物，不因class-source typed投影重写旧map或伪造全类map。完成请求比较raw方法文本+自身map、projected method、whole class分别一致；所有物理BCI仍归原方法。typed候选不成立时，可靠原generic/AST/正文投影不因美化失败而降级；真实Stop则沿现有事务原子回滚。
6. **复用与参考。** 使用既有Rust AST、Signature parser、List模型和标准库，无新增外部依赖。参考JADX LoopRegionVisitor的cast-result与iterable-type配对以及cast时序；不照搬其可变ArgType修订，也不需要引入其IR或代码。完整原始/三方对照和本仓拒绝门比文字相似优先。

typed foreach若是唯一正文编辑，也必须触发投影、发射和事务发布；不能仍只以call casts/removals非空为入口。最终重验必须闭合具体foreach/cast站点或重投影opaque AST相等，不能仅比较每slot元素类型集合。

typed资格是独立的可选约束，不得将不匹配的cast类型直接塞入既有List正文actual-type集合而使原本合法的泛型声明降级。例如 `List<String>` 配raw Iterator再显式cast Integer是合法Java；本片应保留其旧Object头/Integer cast和List<String>声明，不能把typed拒绝误当generic source拒绝。

## Risks / Trade-offs

- 错E或仅依据presented type删除真实cast → 以已接受Signature元素、原始AST cast和物理checkcast三方精确闭合；raw/wildcard/type-variable/不匹配对照保持旧输出。
- 原目标在零次迭代/异常后值变化 → 两种body绑定全部保留，完整原类和循环外可观察状态、错误元素/iterator异常事件控制一起验收。
- class级commit前后计划不一、可选evidence控制路径 → 同次opaque AST、最终闭包重验和default/all/no-rule-details，以及真实late预算/取消/期限门。
- map相对错误文本、header吞掉cast来源 → raw方法text独立边界/UTF8/物理origin审计，typed AST保留cast派生origin；不冒称map对投影文本有效。
- 精确原名头与额外局部复制未清理 → 作为单列后续质量目标，不误报整个黄金断言/71单元完成；此取舍省去无必要的作用域求解器。

## Migration Plan

先root独立接受冻结原/JADX/Jarde基线和未修生成矩阵；再Luna私有实现、root对抗审查后应用。root串行运行Rust/JDK/CLI/JADX/Git/网络，默认stack、至少5GiB空闲、target1GiB每秒守卫，Rust结束exact clean。按tasks完成公开/内部门、完整生成Java双编译器/双VM、邻近拒绝回放、fmt/CI Clippy/MSRV、两fresh native及自身产品CI；失败原样保存，不改旧阈值凑绿。只在真实证据满足时勾任务，提交推送与干净main交接另核。
