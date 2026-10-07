# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **门控复核**：`results/gating/` —— 四判据单独翻转两巡查锚（refused/partial/exit4 → projected/complete/exit0），三负例保 A 文本。落点裁定：`project_class_source_double_brace` 是**分派前置**投影 pass（`anonymous_interface_projection=Projected` 认领文本），A 路径代码与判据逐字未触 ✓。
2. **六处测量更正追认**（`01-gating-and-locate.md`）：`<clinit>` AST 保留第三理由、平台超类**不从输入读**（伴生自身字节陈述合法性——正确保守）、census 第三路径判别、捕获读的 post-call `this` 接收者、转换位锚、以及 **`emit_class_source_anonymous_return` 未抑制 void return 的潜在不可编译文本缺陷**（本片 `<clinit>` 锚暴露并修复）——全部采纳。
3. **root 实测**：DB 渲染双括号源级形（`new java.util.ArrayList() { { this.add(...); } }`——含静态字段 `<clinit>` 位与捕获形 `withCapture`），0 引注；控制套件 `ctor_reorder_dispatch_guard` 2/2 + `fixture_behavior_guards` 7+6 + `double_brace_capture`（A 片套件语义更新）4/4 + 新套件 2+4；回放（ignored）绿；oracle 3/3。
4. **门禁（权威口径）**：全量 exit 0、**338 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0（root 重生成脚本——/tmp 脚本被清重建）；openspec **310/310**。
5. **corpus**：fixtures 899 单类视 0 delta（该视无伴生——文档化）；family 视 4（巡查 DB×2 + 新控制×2）；evidence 2725→1（db.jar DB）；指纹 +20/+105 纯增；census `(879,3829,334,2445,8)→(899,3867,334,2445,8)`。
6. **CI**：合并推送后 run 为准（监控在案）。

## 裁定

- 六边界登记采纳（接口子/空块/块内 return/块局部遮蔽/每法一分配/额外伴生字段）；
- 巡查 README 处置行由本验收关闭（第 12 窄缺口，路径 B 交付）。
