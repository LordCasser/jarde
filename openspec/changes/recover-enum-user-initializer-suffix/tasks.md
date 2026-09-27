## 1. Enum 后缀证明

- [ ] 1.1 复核当前 initializer Code、同次结构候选和字段身份入口；为 Map 字段、values array、增强 for loop 与 put 参数记录精确 class/member/BCI 约束，并用原有 `Measure` 与普通 enum 回归验证旧投影不变
- [ ] 1.2 证明唯一 Map 默认构造、`values()` 数组读取、唯一 loop head/backedge/exit 及 `put(name,current)` 次序；用冻结 CustomInit 正例编译/运行及多 map write、额外调用、异常/未知 loop 负例验证原子拒绝
- [ ] 1.3 在全部 suffix 节点具备可呈现结构前拒绝 enum projection；确认物理 fields、`<clinit>` recovery 和停止/来源报告保留，运行预算与取消边界测试

## 2. 源码投影与验收

- [ ] 2.1 复用 enum 常量组 writer 输出正常 enum 常量、Map 字段初始化和有序静态循环；编译 CustomInit 完整源码及原 API consumer，执行 `java -Xverify:all` 并核对 map key/identity 输出
- [ ] 2.2 对照原 class、固定 JADX 与 Jarde 完整源码的 Java 8 编译/验证运行；检查普通无 suffix enum、DT-11 String 参数 enum 和已有 Measure 单赋值 suffix 回归不被扩展/回归
- [ ] 2.3 验证 OpenSpec strict、来源 anchor 与 class-source reports；重放证据并确认失败诊断稳定、临时 Cargo target 自动清理
