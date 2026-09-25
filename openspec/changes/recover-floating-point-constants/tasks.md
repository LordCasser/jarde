## 1. 固定常量与编译期折叠边界

- [x] 1.1 将36行审计收成最小Java8正面fixture与source-only helper/runner，保存原class/JADX/jarde修前对照、class hash/Code数；root统一冻结永久语料，不能把536项方案实验冒充生产验证。
- [x] 1.2 用冻结输入的精确池/Code补丁固定正负quiet及signaling NaN、canonical NaN加fneg/dneg、真实0/0运行时操作；原class必须经-Xverify:all验证，明确JADX和方案实验的raw-bit差异，不增永久边界class。

## 2. 补齐忠实常量呈现

- [ ] 2.1 在现有facts/decode与AST/类型中接通原始float/double bits及有限叶子，fconst/dconst与池常量均有坐标测试；不采用宿主浮点归一化或新增数值抽象层。
- [ ] 2.2 用统一emitter精确拼写有限值并保留f/d类型，特殊值复用已证明的常量除法写法；完成次正规值、负零、取负词法分组、嵌套优先级及类型回归。
- [ ] 2.3 在既有值构造和生产者路径共享NaN准入及有界常量树判断，拒绝未证明的NaN；真实闭合浮点操作复用已验收的有名值保存，禁止常量折叠改变bits，无法证明位置时明确拒绝。来源保留常量/操作/失败消费者，已有调用效果不丢失，不新增求值器或运行时bit-conversion调用。
- [ ] 2.4 验证默认/完整证据文本一致、真实BCI/成员来源、输出不足及来源不足的commit/replay契约；深度判断不能绕过既有停止边界。

## 3. 实际恢复与主代理验收

- [ ] 3.1 原样重编译实际恢复的完整正面类并对照36行执行；另通过实际恢复路径验证536项有限bits，覆盖重载、NaN比较、canonical NaN运行时取负和真实0/0操作，不能用手写候选拼写替代恢复输出。
- [ ] 3.2 root审查常量折叠与拒绝来源，独立复跑特殊NaN/负零/算术和相邻数值比较、取负、调用及预算回归；统一检查census/fingerprint、fmt/clippy/OpenSpec strict，将范围外字段及NaN传播债务单列。
