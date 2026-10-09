## Context

动机见proposal。子数组candidate CLI SHA `27e53f610f1915c16c21fb144d63aebafb7d376c3eff349b6b5de7673a313123`，精确四源身份见前片results/candidate-cli-v1.json。candidate-root-v4全24腿完成、22成功；独立root1116checks核对闭集/输入/所有成员/生成源/空CP/SP/runtime双流，但状态明确为semantic_replay_passed_signature_pending，子数组任务3.1未完成。两direct腿仅collectionGrid保留ordinary_generic_source_unproved。

静态链见前片results/collection-signature-architecture-audit-v1.md：report.rs::generic_return_candidate根NewArray落入None；class_source.rs::ordinary_parameterized_declaration无法取得同次Program/SSA candidate。Signature::Array拼写已支持递归，Builder现有initializer/store/类型及effects证明已闭合，不能重新做array pass。

## Goals / Non-Goals

**Goals:** 以同次完整Program/SSA的数组创建证据支持参数化返回声明，闭合已定位的无参数、全无界wildcard数组片。

**Non-Goals:** 不推导元素具体泛型实参，不改正文为`new Collection<?>[][]`，不支持一般泛型参数/throws/类型变量/嵌套成员类，不重做assignability或snapshot hierarchy。

## Decisions

### 1. 只补既有返回证据交接中的数组形状

GenericReturnValue已用于Program/SSA到class-source的分层交接；增加一个必要的数组创建形状分支，携带实际数组类型与准确allocation/return来源即可。若既有字段已提供同一事实则复用，不重复构造array/Site计划、不建另一份泛型证明框架。

candidate只接受完整非ragged的单Return NewArray、无形式参数、物理单block/no phi/完整Code、准确areturn及该return的SSA使用。实际创建类型、presented类型与物理返回擦除应一致；实际allocation必须NewArray且形状一致。沿用现有原始来源集合遍历与物理instruction闭集验证，不能只看返回字符串或只看quality枚举。原始initializer以及内部类型/求值/handler闭合由同次Builder已有证明承担；候选不扩大它们。

新路径所有新增循环和类型rank walk使用同一Budget/poll及已有Stop透传，不建立非计费预扫、独立预算、feature开关或大Plan clone。既有collect_expression_anchors在旧member creation路径未逐node poll，若复用只能如实记边界；本片新受输入增长控制的遍历须有实际预算/取消证据，不顺便重做所有旧路径。

### 2. 在声明层证明全无界wildcard与同一数组擦除

JLS8 4.7规定全无界wildcard参数化类型可具体化，且其数组递归可具体化；5.1.9规定同rank的raw数组可转为参数化数组，全无界wildcard无需unchecked warning。[JLS4.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-4.html#jls-4.7)、[JLS5.1.9](https://docs.oracle.com/javase/specs/jls/se8/html/jls-5.html#jls-5.1.9)。本片选最窄关系：数组每层rank、实际leaf和Signature擦除相等，leaf单source段且实参全部Any；不从这一规则推断具体元素类型。

class-source复用reader已验证的Signature/descriptor erasure和既有signature spelling；仅对上述数组candidate、无formal/type parameters/throws的Signature选择这一分支。exact、extends、super、TypeVariable、rank/leaf不同或不完整candidate保持原明确拒绝。匹配通过后只投影原始返回声明，原始NewArray文本不改变，不添加cast来掩盖类型事实缺失。

### 3. 不使用额外库或复制JADX放宽规则

现有Runtime/reader/AST/SSA/预算已足够。外部泛型库、JADX整体类型推断或新的hierarchy服务不能替代同次物理证明，增加维护/许可成本而无收益。旧JADX完整输出可作结果参考，但不把其泛型声明能打印当本引擎正文证明。本片不复制JADX代码。

## Risks / Trade-offs

- [擦除可运行冒称泛型恢复] → 同时检查参数化declaration、marker、真实Signature及全部成员，再完整隔离重编/运行。
- [ArrayList[]直接借Collection<?>[]跨leaf猜测] → 本片只支持实际根数组与返回擦除相等，子元素关系仍由原Builder证明。
- [具体实参、bounded或T[]误接纳] → 同erasure的Signature字节/解析控制及真实initializer必须拒绝，不执行故意变义mutant。
- [局部Program/来源伪证] → 同次实际根allocation/areturn、SSA身份/来源闭集，ragged/missing/错误来源负控及Stop。
- [共享计费影响严格pins] → 实际解释并迁移必要计费，保持P5门槛；旧失败保留，不放宽预算。
- [其它项目持续消耗磁盘] → root只清本仓/可恢复项目历史CLI，20GiB线不降低；空间不足时本地focused不启动，全仓双seed由确切本次CI覆盖，明确不宣称本地完整测试通过。

## Migration Plan

先冻结前片完整六类运行/Signature拒绝基线，再按本独立change派Luna限定实现，root串行focused与双真实JDK完整家族回放。新CLI须包含report/class_source/build/init/Cargo.lock五文件身份；不借四源旧CLI或旧CI。child片3.1直到本片Signature门槛实际闭合才可回验；两片全仓/MSRV/Clippy/OpenSpec和确切组合代码CI全部成功后提交验收、更新71单元与handoff，不宣称整EM18完成。
