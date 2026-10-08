## 1. 冻结差异与架构入口

- [x] 1.1 冻结六个原族及 direct instance/wide slot/T[]/bound/null/多参数/保留别名/继承 owner 控制；交付真实8/23×debug/no-debug 的原class、JADX、基线整类编译、JVM行为与 binder 反射清单、版本/哈希/失败全文；root核对可重放且无原jar classpath污染。
- [x] 1.2 root核对 Program实际发射、字段物理BCI/身份、published method params与后续重发射，交付 architecture-review-root.md；明确无新增pass/parser/IR/fixpoint与跨层反向依赖，并说明参考JADX/JLS的规则及不同之处。

## 2. 实现与针对性验证

- [x] 2.1 在同次现有Program/候选交付路径提取最小 receiver site事实并关联字段完整使用清单；证明 this/direct formal/保留raw local、唯一BCI和声明作用域；针对别名折this、重复/重绑定及来源歧义给出负例，root运行相应Rust测试。
- [x] 2.2 在方法发布后为receiver补齐physical slot参数表，raw字段访问使用字段擦除目标的封闭赋值证明；T/T[]/bound/null/wide/static-slot0/raw local正例与this/binder/未知写者/owner/read-consumer负例通过；不扩泛型方法头，root运行新增和既有字段/构造测试。
- [x] 2.3 覆盖事实扫描/类型证明预算与取消、字段原子保留、一个安全写者不能覆盖其他拒绝写者；root审查停止传播并运行有意义的预算/取消控制。

## 3. root独立验收与交接

- [x] 3.1 root构建候选CLI并冻结hash，重跑四腿完整原/JADX/基线/候选矩阵，核对字段反射与实际方法API分别计数，保留JADX失败；重放前片23字段族与80构造输入无新回退，交付verification-root.md。
- [ ] 3.2 root通过fmt、CI同口径workspace clippy、两固定seed全测试、显式ignored P3/constructor/bound-receiver与strict OpenSpec；提交推送main后按实际最新HEAD核对全部CI（含真正JDK25 oracle）成功，记录命令结果和失败修复。
- [ ] 3.3 更新handoff与71单元账本中的本片准确边界；全部变更提交推送main，不留分支占用；清理root共享Cargo残留并以git/worktree/df证据核对磁盘与主线交接状态。
