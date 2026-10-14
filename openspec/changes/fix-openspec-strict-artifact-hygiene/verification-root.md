# 文档卫生修复验收记录

2026-10-14 已共享应用并验证，当前 3/4；独立提交与其 CI 仍未完成。本记录不证明三个产品缺陷修复或完整 Java 类行为通过。

原始本地 1.14.1 全量 strict 为 341/24（365项），含38条长正文和两份未识别任务。CI 固定 1.11.0 的本轮原始结果为365/0，两种版本结果不可混称。24文件初稿在私有完整副本的两版本全量各为362/3，揭示三个依赖change的MODIFIED场景缺失；失败raw完整保留，未作为成功证据。

最终28文件保留每条原requirement标题、42段原正文及所有既有场景，正文迁为129个按原段落组织的强制场景。父独立逐字重建原正文，核源/candidate SHA、原场景后缀、MUST绑定和WHEN范围；修正了遗漏family Absent、javac9+及未验证发布的收窄条件。四份依赖MODIFIED迁移自身原文，保留其已有扩展行为。

共享应用后，1.14.1和CI固定1.11.0的真实全量strict均365 passed / 0 failed。1.14.1的27个具名受影响对象单项命令全部exit0。真实`openspec list --json`显示`recover-return-through-effectful-finally`及`recover-switch-case-nonlocal-exits`各0/7；14任务只补未勾选checkbox，原编号/正文保持。应用后的28文件SHA再次全部匹配候选。

证据位于 [document-hygiene-root-v1](results/document-hygiene-root-v1/receipt-files.json)：包含baseline、两次private overlay、shared全量、27单项raw及命令记录，候选patch、全文mapping、文件manifest和独立语义审查。最终patch SHA为`19449f7ab8e5d68ffa01ca0e8ae273c926b11d690644be26aff59403dcf18dae`。主实现Rust、夹具和Git index未由本次文档修订改变。missing-MODIFIED-header INFO继续保留，由对应计划处理；没有宣称归档成功。

2.2由coding root串行独立提交并核该提交的CI后验收，当前保持未勾选。
