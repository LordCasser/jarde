## 1. 前置条件与证明边界

- [ ] 1.1 复核已验收的单站点匿名接口投影及其精确 AST、分配扫描、构造器和 owner-XRef 接缝；原样重跑正例、双分配、跨类引用及不完整/预算拒绝测试
- [ ] 1.2 为选定的根分配、一个外层实现方法、一个嵌套直接返回分配和精确内层物理类增加有界关系证书；绑定类型化 nesting/enclosing 元数据、接口合同、描述符与构造 BCI，不使用二进制名模式推断
- [ ] 1.3 只证明冻结形态中的内层 immediate-parent 捕获边：一个 synthetic-final 描述符、一次精确构造器写入且正文无读取；拒绝局部捕获、捕获读取、额外字段/效果、额外分配和不完整 XRef 扫描

## 2. 结构化嵌套投影

- [ ] 2.1 发射内层 child 的完整方法，并从保留的同次 AST 向外组合父方法和根表达式；通过结构化 class-source writer 保持方法声明、BCI/来源锚点和范围翻译
- [ ] 2.2 要求根/child 报告完整且拒绝并存类族投影；只在证明、输出和预算步骤全部成功后组装并提交一个候选
- [ ] 2.3 增加负例测试：内外层重复分配、外部身份引用、`EnclosingMethod`/接口描述符不匹配、读取 `this$0`、额外构造效果、方法/来源覆盖不完整、预算耗尽及取消；断言没有部分投影被发布

## 3. 冻结验收

- [ ] 3.1 为 [DT-07 replay.py](../../evidence/java-syntax-2026-09-27/dt07-nested-anonymous/replay.py) 增加修后模式；原/JADX/Jarde 完整源码和同一 Runner 通过 `javac --release 8 -g:none`、`java -Xverify:all` 且打印 `1`；根源码包含嵌套匿名表达式、不含物理匿名二进制名
- [ ] 3.2 运行相关匿名类/class-source 定向测试、格式检查及适用检查，并通过 `openspec validate inline-proved-nested-anonymous-interfaces --strict`；记录全工作区的独立阻断，不扩大当前变更
