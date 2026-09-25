## 1. 固定异常表达式与拒绝边界

- [x] 1.1 以已有throws审计收成最小Java8正面fixture及source-only负面对照，覆盖null、参数、新异常、调用结果、显式cast、条件、命名catch和checked异常；保留初版finally/synchronized拒绝，不删改生成正文。记录class哈希、Code数量、javap与修前红测试，主代理统一处理census/fingerprint。
- [x] 1.2 固定合法重复消费、旧局部值及athrow较低栈值变体；对patched class用java -Xverify:all确认对象身份和次数，来源断言覆盖throw与必要生产者。用当前真实SSA读集合确认清栈不是额外读取，不凭注释假设。

## 2. 接通已有语句消费链

- [x] 2.1 添加忠实Throw语句节点及既有遍历/发射适配，在最终athrow位置呈现唯一真实异常操作数。以null、参数、条件和catch回归确认Java/Structured、真实源码与来源；不把恢复当作类层级验证，不添加Throwable强转或新pass。
- [x] 2.2 在现有new/invoke/checkcast消费与失败生产者规则中接通Throw，仍保留guard合成重抛所有权；验证构造和调用只执行一次、失败完整引用、较低栈值效果不丢，普通cast/new与TWR/monitor相邻回归通过。
- [x] 2.3 验证默认无来源、真实BCI/成员来源、commit/replay一致与正文/证据预算停止；任何停止不得发布未支付语句或改写已提交正文。

## 3. 三方执行与独立验收

- [x] 3.1 将实际恢复的完整正面类重编译，与原class比较异常类型/身份、调用顺序、计数、生产者自身抛错与checked声明；对照jadx并记录精确范围，不把被拒绝的guard方法删掉后声称原混合类全部通过。
- [x] 3.2 主代理审查最小架构实现并运行独立输入；复跑消费、旧值、guard、来源及预算回归，统一更新语料清单/指纹，执行fmt、受影响clippy和OpenSpec strict，将新问题与既存门禁债务分别记入verification。
