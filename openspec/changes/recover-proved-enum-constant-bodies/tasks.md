## 1. 冻结证据与前置门

- [x] 1.1 冻结两个带专属体常量、一个带体与一个普通常量、两个普通常量的自写 Java 8 `-g`/`-g:none` 原/JADX/Jarde 完整类与 runner 对照，保存版本、字节码来源、重编/运行结果和 SHA；由 root 从冻结源码独立重放，确认当前缺口和 JADX 的正例行为。[Root 复制重放](verification-evidence-1.1-1.3.md)
- [x] 1.2 先完成并独立验收 [recover-proved-enum-constants](../recover-proved-enum-constants/tasks.md) 的两常量主枚举计划：**直接构造主枚举、一个源级整数参数**的普通常量列表、`$VALUES`/辅助成员及用户 `<clinit>` 后缀均通过其原有验证；在本变更定向测试中确认计划未证明时不能单独内联子类体。该结果只是共享候选/证明器的前置，不宣称已证明 `Op`。[Root 基础变更验收与本类拒绝](verification-evidence-1.1-1.3.md)
- [x] 1.3 用 `javap -v -c -p` 固定 `Op`、混合带体/普通常量及 `Plain` 的构造 owner、源码参数数目、`<init>` 描述符与隐式方法形状；证实 `Op`/`Plain` 的零源参数和匿名 owner 超出基础变更准入，并将此差异作为后续正/负测试的显式前提。[证据与边界](verification-evidence-1.1-1.3.md)

## 2. 常量到选定匿名子类的证明

- [x] 2.1a 只扩展 Java 8 两常量的同次候选采集门：`Op` 的 `ACC_ABSTRACT` 不阻止 `<clinit>` Code、全方法 member-use 和精确构造 owner/BCI 被采集，无 Code 抽象声明保留物理头事实；`Plain`/`Mixed`/`Op` 的候选身份与预算/取消有定向测试，Stage/Measure 不回退。此步不产生成功组证明或类源码投影。[Root 验收](verification-2.1a.md)
- [x] 2.1b 从同次常量构造点的精确 owner/BCI，在选定环境和同一预算下读取唯一子类，核对 `this_class`、直接父类、typed `InnerClasses`/`EnclosingMethod` 和唯一子类构造器；测试合法 `Op$1`/`Op$2`、混合 owner、普通零参数 `Plain`、同名无关类、缺/歧义定义及篡改属性。只产生私有待证关系，不凭 `$1` 名称、synthetic 标志或子类顺序投影。[Root 验收](verification-2.1b.md)
- [x] 2.1c 将零源参数与两有序常量的隐式成员核验接入待证关系，显式核对主类 `Op` 的 `ACC_ABSTRACT`/无 Code 抽象声明及 `Op`/`Mixed` 的额外 synthetic 访问构造器记录；当前一构造器/全 Code 成功门不得被全局放宽。正例与不完整成员控制保持 `Refused`；错误构造 owner 不产生待证关系，若底层 IR 因无效字节码停止则如实 `Stopped`。直至 2.2/2.3 的使用、委托与正文闭合后才允许整组 `Proved`。[Root 验收](verification-2.1c.md)
- [x] 2.2a 为现有 P1 `scan_candidates` 增加精确原始 owner 候选过滤：同一物理遍历可找出 `Class`、`Method`、`Field` 的 child-owner 使用及坐标，完整性/预算/取消语义不变。测试精确类符号查询会漏掉的子类方法/字段 owner 与方法句柄，证明该过滤的必要性；不新增索引或扫描器。[Root 验收](verification-2.2a.md)
- [ ] 2.2b 复用所选输入范围的完整 Code/XRef 与同次主类 Code，检查子类只属于一个常量构造，覆盖另一个常量、普通方法、字段/构造器句柄的额外使用及已构造对象的额外别名/存储效果；任一扫描停止、未知候选、引用未解析、选定定义歧义或常量 `putstatic` 后栈值未闭合时拒绝。测试单点通过、多点和未知拒绝，保存命中的物理方法/BCI，而非按 `$` 名字计数。
- [ ] 2.3a 逐边证明子类构造器将 name/ordinal 与纯 `null` 哨兵交给唯一 synthetic 访问桥、访问桥原样转发到主类私有构造器且不读取哨兵；普通常量和 `Plain` 的直接路径按自己的物理构造器核验。以桥改写 ordinal、读取哨兵、额外效果和错误调用目标控制拒绝，不把 verifier 通过当语义证明。
- [ ] 2.3b 证明每个选定子类仅有无额外效果的编译器构造器与可拼写覆写方法，无捕获字段、类初始化或未解释成员；每个有 Code 的方法只恢复一次，要求完整结构、来源、声明及预算/取消一致。以额外字段/副作用、正文 fallback、声明不合法和异常体控制拒绝。
- [ ] 2.3c 合证两有序零参数常量、隐式数组/辅助方法、2.2 独占使用与 2.3a/b 的构造和正文；`Op` 的每个无 Code 抽象方法均由各常量体完整实现。仅整组闭合才发布 `Proved`，缺少抽象实现或任一前置证明时保留 `Refused` 与物理事实。

## 3. 类级原子投影

- [ ] 3.1 在主类已证常量列表的类装配侧车中保存字段/构造 BCI、选定子类身份和同次恢复的目标成员；不解析已发射 Java 文本、不重跑未计费方法。定向测试核对成员身份、来源与每个方法只恢复一次。
- [ ] 3.2 仅在某常量全部子类方法均可发射时，把方法声明/正文写在该常量的 `{ ... }` 中；两个带体常量和混合常量的 Jarde 完整源码用 `javac --release 8` 重编，`java -Xverify:all` 对照原 class 的 `values`、name/ordinal、`getClass() == Op.class`、apply、异常与初始化次数。普通两常量不新增 `{}`。
- [ ] 3.3 投影失败时整项不提交且不删构造器、字段或子类方法；主枚举报告保留原物理成员，子类独立请求仍可追踪 Code，类源码标明选定子类身份。测试 all/essential 文本相同、JSON 来源、method-only 不变，紧预算/取消没有半成品类体。

## 4. 独立验收与资源收尾

- [ ] 4.1 root 从冻结证据独立运行三方 Java 8 `-g`/`-g:none` 对照及所有拒绝控制；审查 Jarde 未采用 JADX 的 `$VALUES`/synthetic 隐藏启发式，执行相关 Rust 回归、fmt、可归因 Clippy、`openspec validate --strict`，记录验收与分离的架构债务，并清理私有 Cargo target。
