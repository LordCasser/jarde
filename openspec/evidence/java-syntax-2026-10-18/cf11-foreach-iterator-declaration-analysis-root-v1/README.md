# CF11 Iterator 空声明：下一项固定差距

仅分析，没有新 OpenSpec、实现或测试通过声明。当前 foreach 原名绑定产品与本分析分开交付，不增加 CF11/71/612 计数。固定完整 TestNestedLoops 输出仍有 Iterator local5 空声明；源码/物理 astore@33/raw map 已闭合，JADX 在变量生成前抑制消费完的 iterator，Jarde 已先发射祖先空声明。

归档保留 Luna v1、v2 更正和 v3 状态流补充、root审查、固定 JADX commit 2fb1b163… 的五个完整参考源码。v1 字面黄金匹配表述由 v2 修正：debug 两头仅归一全限定类型后等价，none保留已有确定性名字；不能称五个 containsOne 已直接通过。add(E) 投影已无包装；equals/remove 可另用准确重载证明去除 Object 转型，不混入祖先声明清理。

现有候选的 slot/reuse/SSA/owner/来源闭合可复用；祖先 Vec 在调用栈，当前 Builder 没有可复用的待删声明状态。若需要窄证书，仅携带跨列表缺失关联，方法/SSA固定上下文不重复建模，纳入既有 checkpoint 回滚并核唯一实际 foreach+唯一空声明、预算/计数/原子应用；不增加通用死变量清理或新的 IR/pass。

本地参考源码是静态依据，不冒称由它重建 JADX binary。归档11成员，SHA256 62b4fdfeb403b50098ab28381019eafd1e3280f2c773922a65c69d31b8b1df5e；原始完整矩阵在 foreach-local-bindings/results/original-root-v1。
