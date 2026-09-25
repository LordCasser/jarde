## 1. 固定比较缺口

- [x] 1.1 将自写三类型六关系/反向关系、int 对照与执行 driver 收为冻结 Java 8 fixture，补来源和拒绝边界回归；记录 class hash、javap、jadx 与 jarde 修前结果，至少直接 long/float/double 测试修前失败。复用已有 comparisons 审计输入，boolean 汇合留作负对照。

## 2. 构造条件

- [x] 2.1 解码五种比较结果事实，复用 SSA 检查唯一、同块、相邻零分支；与既有整数比较区分。用五 opcode 和组合边界测试验证，禁止新增全图索引或比较结果 AST。
- [x] 2.2 在既有 condition 的实际分支极性后映射 Binary/Not；连通必要的延期/失败来源和纯数值循环准入。验证六谓词、两方向、source map、旧局部值与非条件消费拒绝，不能扩大其它区域规则。

## 3. 执行与主代理验收

- [x] 3.1 用真实恢复正文重编译，与原 class 比较 long 边界、两类浮点的 NaN/零/无穷/有限值及左右调用顺序、计数、异常；记录精确执行范围及 jadx 差异，修复后不能照抄其 34 个错值。
- [x] 3.2 主代理复核源码与独立构造输入；执行条件、循环、旧值、失败生产者、来源/预算的相关回归，集中更新新增 fixture 的 census/fingerprint，执行 fmt、受影响 clippy 和 OpenSpec strict，并分别记实测通过与既存门禁债务。
