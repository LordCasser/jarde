## 1. 同源声明交接

- [x] 1.1 从 raw_facts 已读取 Header 向既有只读 MethodDeclaration 传递 raw class name/class flags，并保留物理定义来源；验证 getter 反映真实 Header、class/member flags 不混淆、同名不同物理定义不串用、停止路径不凭请求补造声明。
- [x] 1.2 在 facade::recovery_facts 适配到既有 DeclaringClass 并使用安全名称显示；通过公开 Engine 验证普通实例/static、interface default/static、constructor 和 clinit 的声明 form。删除适配的变异必须使公开入口测试失败，不能只测试低层 builder（A10/A16）。

## 2. 范围与回归

- [x] 2.1 对普通无 accessor 方法比较改前/改后 Header、Body 和 I/O usage，证明无新增扫描；覆盖非法名字、低层缺事实、早停/取消和 abstract/native 无 Code，保留其适用与停止语义。body 中仍有 fallback 时不能因声明补齐升级 quality（A13/A14/A16）。
- [x] 2.2 运行公开恢复、声明规则、CLI、R8/R9、source-map 与 X0/X1 隔离回归及 fmt/clippy/OpenSpec strict；更新已补齐的类级声明字段说明，明确其它 metadata 仍未支持，记录固定 SHA 与验证结果后同步/归档（A13/A17）。
