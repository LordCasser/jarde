# root 集成检查，尚未验收

Luna v2 补丁经五份 base/result SHA 与 patch SHA 核对后集成。root 修正 pure-essential 测试的 recovery evidence 字段、删除只为测试引入的单项公共 wrapper、批量调用改用既有 batch 入口；没有 indexed get 时跳过新增 AST 遍历，修正文档 child 方向。随后实际 focused v1 的指纹重建/校验共6通过（1旧忽略），report 两项 batch 对抗测试通过；root lib 因测试缺 CancellationToken 导入失败。root 改用完整路径，保留失败日志。

focused v2 的3个旧 List 单位门通过，新正向 List.get 用例返回 None。diagnostic v1 先直接验证 AST Cast(Call) 正向通过，再打印物理证据：voidGetCastString 参数读取 [(0,3),(0,12)]，receiver census 仅 [(0,12,14)]，漏了 load3→size4。旧实现固定 Slot::Stack(0)，但 i < values.size() 的 i 在栈底，接收者位于 Stack(1)。这是真实覆盖缺口，不能移除 size 或放宽物理用途闭包来绕过。root 已要求 Luna v3 修现有 census 的 receiver 识别，沿用同次 SSA/实例调用/descriptor 事实；实参不能混作 receiver。无需新 IR/pass/载体。

三次运行均已受 5GiB/target1GiB 守卫并 exact cargo clean；未称测试全过或产品交付。诊断源中的临时详细 panic 待根因修复后收窄，最终源需重新绑定。

独立静态审查的两个预算点是 HEAD 既有 generic List proof：复杂度乘积被计入 IrItems、过滤前缀缺逐行 poll。本片保持边界，单列后续债务，不据此混改旧算法。双序 batch 的保守拒绝尚无固定 javac 反例，保留说明。

## 最后有效结果

Luna v3修复真实Stack1接收者库存后，root补充其descriptor分配/set比较收费和每行poll，focused v3 40/0/1及相关广门578/0均接受；最终CLI90b30ad…1f64c完整32Jarde编译/64VM832case全同原。精确差分承认成功marker移入method头下，以及legalWrongCast/wildcardList进入既有统一拒绝尾部的两条精确说明变化；没有扩大允许的代码或raw/map差异。详情见results/README与最终归档。旧预算债务仍单列，不混入。
