## 1. Evidence and proof

- [x] 1.1 固定四组 retry-v2 `exc()V` 的真实字段/Builder/String/Error value flow、refusal、异常覆盖和来源；归档二轮审计，核数据与40命令对照一致，不把观察当实现通过。
- [x] 1.2 在现有 concat 入口复用 field::Plan 与物理code，为直接实例字段operand核receiver Load/下一append/全方法单使用、完整tail唯一消费者及覆盖；永久正反例与新增扫描内部budget/cancel无部分owner测试通过后关闭。
- [x] 1.3 用既有AST对新field-backed链的Object片段显式保留valueOf(Object)即时转换；永久测试核null/char[]真实overload与来源，不改旧已接受链文本，含toString修改后续字段的双JDK运行门通过后关闭。

## 2. Integration and delivery

- [x] 2.1 原/JADX/Jarde双JDK双profile retry-v2与父项完整类重新编译运行，Error类型/消息/计数/事件全部同原；生成源码不得修补，CLI/输入/40实际命令与raw独立校验。budget/source-map覆盖新增expression真实BCI。
- [x] 2.2 新永久回归、fresh双seed workspace、fmt/当前CIClippy/global strict、5GiB/target1GiB/1秒守卫/default stack/finally clean全部实际通过；若与父项共享门禁，来源和测试闭包必须包含两项最终代码及fixture。
- [ ] 2.3 精确产品提交推送，自身CI接受后更新父项完整类门、handoff和账本，并核当前main干净/targetabsent；不因此增加整71/612或CF18完成数。

2026-10-12：1.1–1.3/2.1 共4/6已按 [integrated-comparison-root-v1](results/integrated-comparison-root-v1/README.md) 的真实focused 390+91、父项40命令/12全类同原与字段24命令/8全类同原及来源门关闭。完整工作区与自身产品CI仍待完成。

2026-10-13：fresh workspace-v7 第76批发现 WA.main 的旧完整文本预期仅有一行字段Builder拼接差异；九accessor正文与其他字节相同，同批生成全类重编译和原运行对照已通过。保留历史fixed样本，只对该println做一次显式literal适配，继续全文exact比较；fresh focused-v11加完整9项访问器target，随后须全新双seed workspace，2.2/2.3保持未关闭。

2026-10-13：fresh workspace-v8 reader lib最后population tuple发现未计入本轮23个class。独立核全部1140旧class恒同及新增139body/27handler/138target/0subroutine后，仅更新总量和ledger，结构断言不变。fresh focused-v12含完整reader lib，共705成功/0失败/4ignored，CLI字节恒同整类对照候选；见 [局部验收](results/reader-census-fix-and-final-local-root-v1/README.md)。全新workspace-v9已运行，2.2/2.3继续保留未关闭。

2026-10-13：2.2 已按fresh focused-v12和完整workspace-v9关闭。八focused命令705/0/4、当前CI Clippy/global strict通过；两seed各96批/364targets、合计7050成功/0失败/196ignored，195命令全0、4854来源恒同、5GiB/target1GiB/1秒守卫/default stack/finally exact clean/target absent均独立核验。见 [完整工作区验收](results/full-workspace-accepted-root-v9/README.md)。自身产品CI和真实HEAD干净main仍待完成；不增加CF18/71/612完成数。
