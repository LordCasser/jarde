## 1. 基线与设计

- [x] 1.1 root实际独立验收ArrayFill三边界39命令/8完整腿与205闭合文件，确认CONST_INT呈现差距，原/JADX/Jarde完整重编与raw一致；证据baseline-verification-luna-v2.json。
- [x] 1.2 核原候选/AST retention/transformer/use handoff、Emitter commit-replay及writer span，记录两份只读audit并完成本design；不借JADX二进制内部路径结论。

## 2. 最小实现

- [x] 2.1 Luna准备private patch，扩展同轮capture及直接一维int数组Return叶投影，保持唯一候选/名称遮蔽/原body一致性；root审后应用，真实Rust用例证明CONST_INT可见且switch回归不变。
- [x] 2.2 复用body-only emitter replay核精确BCI/name segments，内部handoff新数组body range经既有writer映射；真实测试核重复名称不同BCI、token范围、Field/MethodPoint、歧义拒绝与Stop传播。

## 3. 对抗与完整类回放

- [x] 3.1 构造完整类的同值歧义/parameter与local遮蔽/不支持表达式和数组形状/预算取消控制，默认-all正文一致；永久Rust测试及root对抗审查通过，不把early-stop用例冒称late-stage动态注入。
- [x] 3.2 root冻结新CLI/meta/product pins，完整ConstantIntArray及控制类集双JDK重编-Xverify/raw对原oracle，并验全部physical成员/BCI/来源；不得手改generated源或借原helper。

## 4. 主线验收

- [ ] 4.1 资源守卫满足后root相关fmt/Clippy/OpenSpec strict/reader与fingerprint通过，提交产品后独立确切CI双seed/JDK25/MSRV/fuzz/supply验收，不能借实例片CI。
- [ ] 4.2 提交推送main并更新handoff/71账本实际范围与任务；只清本仓编译残留，保留历史失败、源/class/raw及冻结CLI，不冒称EM18整单元完成。
