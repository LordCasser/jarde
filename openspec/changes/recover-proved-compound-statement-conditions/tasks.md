## 1. Frozen evidence and local algorithm

- [x] 1.1 root冻结本地JADX HEAD/IfRegionMaker源码、Switch2真实class/javap/trace及候选CLI完整失败；核对四测试、共享118和两个出口131/136。验收：pins和原始记录可复放，root说明末位e笔误，不将候选join证明算作整类通过。
- [x] 1.2 用保序出口对齐规则核验真实四测试归约，并记录必要表示与现有复用边界。验收：条件树每个canonical测试叶子恰一次、完整解码边可核，payload/return不进入条件测试集合。

## 2. Bounded proof and ordinary statement building

- [x] 2.1 Luna准备私有条件闭合/归约候选，root审查后串行集成。验收：真实解码边、唯一入口、无环、两个出口、完整canonical行与frame/path/scope均闭合；没有BCI拓扑假设或未收费扫描。
- [x] 2.2 增加最小CompoundIf证明载体并审核所有Region汇总、归属、递归、单If专项消费者。验收：全测试及两个arm各有唯一物理owner，单分支专项不能默许额外测试；实测Switch2单If包围CompoundIf的共享void-return有限接续按design4a闭合，编译和相关owner tests通过。
- [x] 2.3 复用现有test_expr/依赖和类型规则构造一次AND/OR/反转表达式，输出普通If AST并保留lead和准确来源。验收：真实Switch2恢复及完整来源、default/all正文/map相同；builder拒绝没有省略独立effect或部分产物。
- [x] 2.4 对外部/重复/非Normal边、cycle、clone/case/frame跨界、内部独立语句及不可无重复归约形状增加必要负向覆盖。验收：有界拒绝保留全部物理block/来源，没有visited泄漏或部分源码。
- [x] 2.5 在真实新增proof和builder构造阶段分别触发预算/取消，运行既有条件/loop/switch/value回归。验收：原因、维度、位置、usage准确且binding/text/map原子缺失，相关旧回归通过。

## 3. Complete source comparison and delivery

- [x] 3.1 root编译执行有副作用、异常路径和浮点NaN的受控完整类，对比源码、JADX、Jarde及调用日志。验收：Jarde每条测试路径与原class调用次数/顺序、异常位置和结果相同，JADX差异独立记录，不更改原check。
- [x] 3.2 冻结同一新CLI，重新比较十个CF12原类/JADX/default/all，核验Switch2状态矩阵及FT2控制、完整生成类集合和物理来源。验收：Switch2与FT2整类真实通过；环境不可用单列，不能用跨版本或其他JDK结果替代。
- [x] 3.3 完成root格式化/适用测试lint、OpenSpec strict与自身产品CI独立验收，更新两个变更verification及handoff后提交推送并清理target。验收：实际命令/raw/pins可追溯，main干净、无功能分支占用，不超额宣称CF12或71单元完成。
