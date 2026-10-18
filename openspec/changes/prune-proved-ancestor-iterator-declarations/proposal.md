## Why

固定 JADX `TestNestedLoops` 的两层 foreach 已恢复，但完整 Jarde 方法和类源码仍保留一条 `java.util.Iterator local5;`。迭代器初始化已被成功循环消费，该无初值声明却已提前写入祖先语句列表；当前候选只能删除本列表中的行，留下了有物理来源、可以进一步证明失用的临时声明。

## What Changes

- 在已有 iterator foreach 成功提交后，仅清理其完整消费的祖先无初值声明；还需闭合整个 `LocalVariable` 的全部物理读写，不能用单个 SSA 值的使用闭合代替。
- 缺少精确变量、声明所有者、真实词法祖先或唯一已提交循环证据时，保留声明和已有可靠 foreach。预算、取消、期限停止沿已有契约传播。
- 完整重编译并运行原源码、JADX 与新 Jarde 源码，核对行为、物理指令来源、raw 方法与类投影；加入同变量其他写入、同槽复用、兄弟作用域和回滚边界。

前置产品 `0a7e3c98f63b128a13a658a475b9e69a9e60d048` 的自身 CI、完整对照及干净主线已接受，当前文档 HEAD 为 `b45167b72486e22f942985340792dd730b308c56`。已有固定原始类是红文本基线；新控制家族还需实际编译、采集和分类，规划不等于通过。

非目标：不扩大 foreach 准入，不删除有初值或仍有用途的声明，不改变 typed 元素/首句绑定，不去除 equals/remove 的 Object 转型，不修历史同列表 gap 时序债务，不增加通用死变量 pass、公开证书模型、IR 或类型推断。CF11 整单元及 71/612 计数不因本片增加。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已提交的 iterator foreach 完整消费某局部变量时，允许并要求清除唯一失用祖先空声明，保留所有不完整、外部使用和停止边界。

## Impact

主要涉及 `crates/jarde-java/src/build.rs` 的现有候选、声明事实、构建回滚与最终 AST 提交，以及已有公开 foreach 回归和完整类对照。只补缺失的私有跨列表关联；方法、SSA、reuse、名称和声明计划继续作为既有上下文使用。无新依赖或 API，继续使用本仓 Rust 工具链和已固定的 JDK/JADX 对照工具。
