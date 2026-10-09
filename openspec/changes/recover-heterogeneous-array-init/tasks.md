## 1. 取证与架构

- [x] 1.0 历史巡查冻结 CT 的 anewarray Number 与 Integer/Long store 及同构锚；验证保留原始 Java/class/hash 证据，不把历史源码片段当作完整行为验收。
- [x] 1.1 固化当前 producer/consumer、现有平台谓词、Runtime header walk、store SSA 与 JADX 效果边界审计；验证当前文件与报告相符，不使用过时 Atlas 行号。
- [x] 1.2 归档双 javac 完整 baseline、合法修正版和冻结 CT.class additive 重放；root 校验全部文件/双流 hash、隔离 argv、完整 source 集及实际输出，不以 exit 0 计成功。

## 2. 实现与边界

- [x] 2.1 初始化器复用所有现有标量/数组兼容事实并支持已知引用关系的等秩提升；focused 测试证明类型方向、primitive/rank 边界和无元素 cast，调用参数旧行为不变。
- [x] 2.2 在同一 snapshot producer 为真实 aastore source/component 生成准确 BCI 证明，支持标量和等秩自有类数组，修正载体注释；验证 direct/两跳/interface、非零 stack prefix 与错位/缺失/深度/预算控制。
- [x] 2.3 添加双 javac factory-element 完整正例与独立 direct-new 未覆盖控制，保留 CT 与所有自有辅助类；验证同型/null/Object 与效果次序/非法方向零回退，README 记命令及 SHA。

## 3. 独立验收

- [x] 3.1 root 构建并冻结 candidate CLI/source/hash，在同一 baseline 输入上重放全部正例与负例；全部生成类隔离编译并 -Xverify:all 运行，与原程序/JADX 双流对比，所有实际失败保留。
- [x] 3.2 完成双 seed workspace、MSRV、fmt、CI-exact clippy、必要 ignored gate、strict OpenSpec、fingerprint/P5预算与 diff check；每项保存实际 exit/双流，不因预计通过打勾。
- [ ] 3.3 root 对抗性审查并核对所有上述契约后更新 EM-18 账本和 handoff，提交推送并确认确切 SHA 的实际 CI；完整家族未通过或缺证据时不得关闭登记行。
