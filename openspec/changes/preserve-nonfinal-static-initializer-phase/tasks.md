## 1. 基线与阶段规则

- [x] 1.1 root独立核验literal和ordered两份fresh完整类基线共16腿/62命令，固定全部source/class/raw和物理成员/BCI，确认语义成功与呈现差异。
- [x] 1.2 对照当前静态组/projection与JADX ExtractFieldInit及JLS4.12.4/12.4.2，明确final阶段边界和非final修正，设计文档不引入新机制。

## 2. 最小准入修正

- [x] 2.1 复用已经核对的field flags只对final保留常量RHS阶段拒绝；root审最小diff、受影响静态投影测试通过，其他证明不变。
- [x] 2.2 增加非final常量加有序数组正例及无ConstantValue的final运行时常量拒绝例；静态/接口组、完整表、次序与预算取消回归通过。

## 3. 独立完整类与主线验收

- [x] 3.1 冻结重建CLI对literal/ordered原样完整源码双JDK重编-Xverify/raw对照，确认ordered整组声明提升、无重复static块、nonfinal flags与无ConstantValue、物理方法/BCI保留。
- [x] 3.2 root对抗审查及相关Rust/Java测试、fmt/Clippy/OpenSpec strict/reader/fingerprint和确切新产品CI验收，分开登记实例提升与小栈限制。
- [x] 3.3 提交推送main并更新handoff/71账本及任务实际状态；磁盘清理只限本仓target，source/class/raw与冻结CLI保留。
