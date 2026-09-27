## 1. 来源关系与证书

- [x] 1.1 检查 parent/child 的 class identity、版本、可见性、member flags 与 InnerClasses 两侧行；针对缺失、重复、冲突、非 enum `$` 顶层控制补固定拒绝断言，并以 class-file fact 单测验证关系解析
- [x] 1.2 在同一 class-source 请求的 selected environment 内唯一解析嵌套 enum family，并递归验证 fixture 所需的两层 owner 路径；测试重复定义、缺少 child、错 owner 均拒绝整棵子树
- [x] 1.3 对每个成员 enum 复用完整 enum 组证明；以截断字段/初始化、错误 enum flags 与预算停止验证不得产生局部成员声明

## 2. 根级源码投影

- [x] 2.1 在父类型声明的真实词法位置递归发射已证 enum，保持 `Outer.Inner` / `Outer.Inner.Deep` 的 Java 名称；验证原始 API consumer 可编译
- [x] 2.2 为根级派生声明关联准确 owner/child class 和 enum group anchor，同时保留 child 原始物理 class report；验证物理报告查询内容未被投影替换
- [x] 2.3 以非 enum `$` 顶层类型、普通 enum、不实现接口的 enum 和 `enum implements I` 对照验证不错误嵌套或改变声明头

## 3. 端到端验证

- [x] 3.1 固定两级嵌套 enum Java 8 fixture 与 API consumer，重放 original/JADX/Jarde 完整源码编译和 `-Xverify:all`；要求可支持的原始 API 运行输出一致
- [x] 3.2 扩充三方对照的拒绝边界：缺失/歧义关系、enum 组拒绝及 budget/cancel 停止不得发布部分层级，并验证 class-source 与 report 的停止状态
- [x] 3.3 运行 OpenSpec strict validate、相关 class-source 和 enum 回归测试，检查 Jarde 文本、origin anchors、物理 class report 与负例结果
