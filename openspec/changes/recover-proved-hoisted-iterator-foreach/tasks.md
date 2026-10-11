## 1. 固定证据与后续首拒

- [x] 1.1 接受固定TestNestedLoops既有完整三方基线、同IR-v3准确首拒及声明所有权架构审阅。验收：旧基线wholeclass重编运行同原、inner42与outer13明确分开、LocalVariable/placement/Object头/cast事实可定位；不冒称当前CLI新对照或后续门通过。
- [x] 1.2 用当前冻结CLI重放固定wholeclass并受控诊断声明owner及越过声明门的首拒。验收：同class/Program/Builder事实列出init/test/next准确BCI、LocalVariable、RegionPath、placement/at_region及后续Phi/uses/effect verdict；诊断源码按字节恢复、完整raw及guard/clean，不将临时放宽当实现。
- [x] 1.3 冻结必要控制源码/class并执行原/JADX/当前Jarde双JDK完整基线。验收：每次outer进入的iterator创建次数、hasNext/next/cast顺序、空/null/错误元素异常、written-gap在初始化和hasNext抛错前后观察的局部状态记录完整；未生成类只按真实拒绝验收。owner/split/额外消费边界由2.2永久反例落实，不把控制基线当实现门通过。

## 2. 最小实现与永久回归

- [x] 2.1 根据实际首拒派发Luna私有最小候选，root审查应用。验收：仅复用同LocalVariable完整可见owner、精确Phi input/use occurrence对账与现有有界replacement；匹配Phi已替换/无reader/各arm同iterator，无关Phi不增加门；准确空声明在self.stmts时保留原同列表路径；owner前缀只证可见性，不代替list匹配。新祖先路径间隔只有空声明，删除位置限当前列表、其它列表祖先声明保留。原同列表路径与完整其它循环proof保持，新增工作明确计费/Stop传播；新通用证明机制先拆债务。
- [x] 2.2 增加必要永久fixture/API/owner、split身份、Phi multiplicity/mixed/live/无关Phi、额外消费反例并核真实Stop。验收：default/all/noRuleDetails同一正文决策，全部physical BCI/UTF8来源合法，outer Assign/generic边界不越界；真实AnalysisSteps或其他实际dimension/usage/location和当前方法原子无产物，pre-cancel单列。

## 3. 完整验收与交付

- [x] 3.1 冻结当前完整输入闭包新CLI并root接受完整双JDK三方对照。验收：同源完整generated class未经编辑重编执行，exit/stdout/stderr/身份/顺序及异常同原，来源default/all准确；JADX差异、未恢复部分与原始失败单列。
- [x] 3.2 完成fresh全工作区、fmt/同CI Clippy/global strict与最终CLI绑定。验收：实际metadata各目标恰好一次、逐named outcomes核验；默认Rust stack、5GiB free/target1GiB一秒守卫、最终clean，CLI与实测candidate字节一致。
- [x] 3.3 提交推送产品并独立接受自身CI。验收：确切head/run全部jobs/steps和双seed目标/named outcomes同本地，失败修复后用新产品自身CI，不借前片。
- [ ] 3.4 更新账本/verification/handoff并交付干净main。验收：实际HEAD/main/origin/remote一致、仅main、aux checkout detached干净main祖先无target，关闭文档推送后按真实HEAD审计；仅关闭该窄片，不增加整个CF11/71/612完成数。
