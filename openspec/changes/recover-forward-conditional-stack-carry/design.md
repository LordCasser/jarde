# 设计：前向条件栈值的有限 reader 证明

## 状态与前置裁定

root 已在 frozen admission class 上读取真实 classfile 指令、控制流、SSA 定义/使用与 `replaced_by` 关系，确认 rejection 的 producer、phi、reader 及其执行区间，并证明同值退休栈 phi 的重复 use 与 `getstatic` consumer 计数拒绝之间的因果关系。有限证明已接入；focused、完整类矩阵和时序对照已有实测验收，具体输入、边界与结果见下文。全工作区验证与最终交付仍待完成，局部验收不替代这些任务。

## 候选接受条件

候选证明逐一绑定具体物理方法、原始 `MethodId`、控制流块、SSA ValueId 与 slot，不以名字或固定 BCI 识别：

1. producer 的物理 SSA 类型必须为 `Value::Int`；对象/StringBuilder receiver 留给既有 construction/concat 完整链证明及原拒绝路径。既有 Folded 条件值路径已经证明一个单独、无循环的前向 diamond 与汇合点；不得新增条件表达式识别或一般块合并能力。
2. 目标 SSA 定义是该汇合处同一个 `Slot::Stack(_)` 的 phi；其输入集合必须恰有两个 Value 输入，两个输入是同一 `ValueId`，且该 phi 已被现有 SSA 替换/退休事实证明不再是独立活值。
3. 从生产值剥除这两条等值 phi 边后，只允许一个真实物理指令 reader；以 instruction BCI 和 reader 所读 Value/Slot 身份证明。不得把 phi edge 当成可忽略的一般 use。
4. 前向路径必须有限并可证顺序。producer、diamond 两腿、join、reader 之间不得出现回边、未处理的循环、任何覆盖相关区间的 exception handler、额外 reader 或路径不一致 side effect。
5. 不接受输入 Value 不同、不同 slot、多个 phi、混合或仍 live 的 phi、alias/替代值链扩张、多真实 reader、未知操作、额外 join 或跨多个条件区域的候选。任何证明缺失、冲突、超预算或取消均保守拒绝。

候选在完整 final-consumer/join/dependency 检查成功前保持 provisional，不影响全局 consumer census。

最终落地时，现有普通 consumer gate 仍对非候选值执行原始规则。新规则只可将候选两条经认证的同值 phi 边从 census 视图排除；不得全局更改 `SsaTable::uses`、Value 读取 API 或其他消费者。

## 呈现与顺序

成功候选继续通过已有 Folded 条件值呈现路径，不能引入第二条 Java 输出通道。生成表达式必须仍绑定一个唯一 reader，且物理 map 标记 diamond 两臂、join phi 与 reader 的原始来源。侧效探针应在分支前后安置可观察操作，确认条件求值及唯一 reader 的执行次序与原 class 一致；不得从 join 反向搬动表达式，也不能把可能抛异常的 producer 移到原控制流之外。

## 失败原子性与资源

证明属于 owner commit 之前的预算化步骤。预算停止或取消时不得发布条件表达式、局部声明、方法投影、部分来源映射或消费规则缓存；先前已完成的 Header/Code/IR 与已提交 artifact 保持；未提交的本次正文沿既有 stopped/NotProduced 合同保持空 text/map/declaration，不在停止后补跑 raw recovery。测试应使用实际新的 candidate path，并从 observer 或 ledger 证明 stop/cancel 落在证明执行期间，而非仅在最终渲染阶段触发。

## 负例和回归边界

保留 BI/boolean-loop 完整输入，冻结当前产品的恢复或拒绝状态、正文与来源；该例的回边/循环事实必须使新入口在候选构造阶段拒绝。另保留构造负例覆盖两输入不等、第三 phi 输入、extra reader、live/mixed phi、handler 覆盖、回边与超预算/预取消。复用已有 helper 与真实 classfile/SSA fixture，不建立通用 classfile 测试框架。

## 验收边界

使用原 13 方法完整 admission 类和 Runner 对照；基线输入的 source/Runner 字节、accepted class 文件、45 行基线 oracle 与 33 行 admission oracle 均由 frozen manifest 校验。通过条件为原定义的完整双 JDK × debug/none × default/all 16 个 Jarde cells 全部满足各自预期，以及 45/33 行 oracle 与 JADX 编译/运行真实结果符合其冻结分类。BI 既有状态、正文和来源必须逐字相同；前向执行顺序、预算停止、取消和完整 source-map 必须独立断言。

## 2026-10-14 root 实际裁定

持久诊断 `consume-diagnostic-v1/1.stderr.raw` 已实际完成 SSA/CFG dump：顶层恰 `[If, Straight(join)]`，branch=0/branch_bci=4，两臂7/11，join=12；getstatic0 的 Value4 有 join12 两条 None 与 iadd12 一条真实 use；Stack0 的 Value2 Phi 输入精确 Value4/Value4，replaced_by=Value4 且旧输出无 uses。iadd 后唯一终端为 putstatic13。诊断运行成功，finally cargo clean 已实际运行；收据封装因缓存路径相对 repo 的旧假设失败，故保留 raw 与失败 launcher，不将其视为完整验收 receipt。root 独立 final-matrix-v1 审计 rejected，8个 admission Jarde 编译失败仍保留。

实现限无异常表、无克隆/回边的顶层 `[If(prefix空), Straight(single join)]`。复用 `prove_conditional_value` 与 `prove_carried_conditional_use`，同时绑定既有 Folded phi。新私有证书记录具体 producer、直接 consumer、join；不修改 crosses_a_proved_cut。消费者/terminal/声明三门消费同一证书，终端仍须在 join。只在既有依赖 walk 明确有界时将 producer 保存于条件之前，保留字段旧值及初始化/调用/异常顺序；不把它搬到 join 后重新读取。原13方法 fixture/Runner不变。

现有 reader/SSA/条件证明/绑定与预算库足够，无新增依赖、许可证或维护负担；不自建classfile解析器。

## BI 当前基线的实测纠正

原规划沿用了2026-10-05旧计数实验的“BI拒绝”快照。root实际冻结CLI v6与v11，分别重放recover-conditional-rhs-field-compound两编译器class（SHA d2a54a0f…/884d0220…）及原历史bi.jar class（SHA a5ec6c7b…）：当前v6已由既有布尔field-compound路径恢复earlyRet为Java/Structured。v11在default/all下完整类正文、方法raw map与身份逐字不变。首次比较因强加已不存在的旧拒绝句而失败，4个render原raw保留；修正比较固定当前真实状态，没有改产品或输入迎合断言。新私有carry证明仍对实际BI回边返回None（420库测试中实测），不扩大循环准入，也不撤销其他已验收路径。

focused-v11 root独立协议验收447/0/1，4876源码/fixture闭包未变，峰值888367574<1GiB，exact finally clean/target absent。完整matrix-v2主审计accepted/errors空：16个Jarde整类compile/runtime，45/33行oracle；JADX baseline8失败无runtime，admission8成功同33行。完整矩阵归档已经逐成员复核；fresh双seed、额外CI runtime门与自身CI/最终交付仍待闭合。

Stop合同裁定：该proof在第一次emit前。report::stopped的既有正文合同为NotProduced（空text/map/declaration），不是在取消后另造完整raw fallback；已有generic overlay事务中的已完成raw artifact仍原样保持。新证书Err与builder的?传播遵循此合同。规划中的原“完整raw fallback”措辞已纠正，没有新增回退通道或降低Stop门。


## workspace 回归与修复后的局部验收

v5 第70批和 focused-v12 保留原 ICN whole-refusal 断言失败；v6/v11 同物理输入对照证明 BCI16 的 receiver 从三consumer整体拒绝变为 fragment。v13 的双编译器物理SSA测试核实该值为 Ref、uses3、四块，并实际证明旧同值carry前提匹配；新整数边界拒绝对象receiver。focused-v13 实际451/0/2，包含全部420库测试和原ICN负例；旧v6与v13在两编译器/default-all八次真实render完整正文逐字一致。新矩阵/BI/双seed/自身CI仍须重新绑定v13，不能沿用旧冻结验收。
