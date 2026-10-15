# 历史证据与接手边界

本目录保存本次规划的输入副本，未实施修复、未运行 Cargo/JDK/产品 CLI/GDB。所有实施任务仍未勾选。目录可整体移交；`evidence-copies.manifest.json` 逐文件记录原绝对路径、副本相对路径、bytes/SHA256，历史 JSON/脚本内绝对路径原字节保留，不改写为一次新的执行。脚本只是审计材料，不能在新工作区直接执行并覆盖旧证据。

## Evidence inventory

- `probe-v1/`：原 `/home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-probe-v1/` 全部普通 CLI raw/receipts、分类与调试材料的字节副本。原 probe export manifest SHA256 `9d97f11e8cac082d612ae5d73b7776d0e0acd2ef408a04e1d319337440b8895e` 的41个唯一 payload（451427 bytes）已逐文件核验。wide 与 caps1/2/8/14/15 共六次普通执行，只改变 class_headers；它们是缺陷/对照观察，不是六项修复测试。
- `independent-raw-audit-v1/`：原 `/home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-independent-audit-v1/` 的 raw/trace 审计、脚本、文本差异及 manifest；原五轮 audit 与后来 trace-review 保持各自历史结论，没有倒写旧范围。
- `independent-contract-review-v1/`：原 `/home/lordcasser/.cache/jarde-review/20261015/class-source-proof-stop-contract-review-v1/` 的 review.json/README，42 项输入 pins 与契约/调用路径/classfile 事实。原 review SHA256 为 `101cc6b6fae7c8116bacfc5459ff5eb172d004cdb7c2b5568d4d50aa8a4bff60`。
- `fixture/`：真实 frozen `hier-fam.jar` 和 H1/H2/H3/Helper 原 Java 文本；另保留同家族既有 negative/depth JAR 供后续控制选取，不声称这些附件本轮已执行。公共观察始终使用 hier-fam.jar，SHA256 `1ee63f3836a5e514cea57b7349dd515ca3f98550154beff187fdca4d019105b0`。
- `source-context/`：从固定 c814 commit 只读导出的完整 facade、budget、CLI task、JVM resolver、Java report 快照和标注行范围；manifest 给出 Git blob 与全文件 SHA。它们是源码上下文，不能替代历史编译 receipt 的完整 source closure，也不是当前私有 checkout 编译。

冻结 CLI bytes59602720/SHA256 `cdf06694fafcb4e8cef5a4b9452676489e8cf02495c0254ce38a21e44ee4c7d5` 及 emitter-focused-v2 完整 build receipt 保留外置，精确路径/hash 见清单。两个独立审计已记录其 c814/build 关联，本片没有重新构建产品，也没有把当前 live 全源码套成历史编译输入。

## Confirmed and excluded scope

cap8-v3 的五次 Budget::exceeded 中，前四次经 snapshot prebody proof，第五次经 double-brace postbody。cap15 trace 只有一次 double-brace 拒绝、无 snapshot 栈，最终 class/main 仍 Complete 且 main contains_statements。两有效 trace 的全 JSON 仅去 elapsed_millis 后等于各自正常请求；cap15 main report 去 elapsed 后等 wide，main/class 正文逐字同 wide。trace 脚本只断点/backtrace/continue，没有改变量/返回或注入假 Error；未打印每次具体 BCI/owner 或 Result，不把多次命中冒称多个唯一站点/Err catch。

`probe-v1/gdb-class-headers-8/` 是无效 debugger 对照：run 只有重定向，重置请求参数，inferior 为 cli_request_json EOF/退出1、零 quota 断点。GDB 自身退出0不代表产品成功，这组完整保留但排除；v2 栈较短不能独自定位完整 snapshot 链，最终前置归因采用 v3。cap14 没有自己的 trace。

只确认 snapshot 与 double-brace 两条公共消费路径；reference-overload、direct-parent accessor/field 三 wrapper 仍是静态债务。H1.main 无接口特殊调用，因此狭义 interface-super 计划仍独立且未实施。取消、期限、其他原始预算原因、直接 Err 渠道与停止后 no-work 的永久测试均待实施；不能由历史 ClassHeaders 观察宣称它们已通过。

## Handoff

接手者先核验 `planning-export.manifest.json` 的全部 payload bytes/SHA（该清单自排除），再读 proposal/spec/design/tasks。永久回归使用真实 frozen fixture 与公共库/CLI，不依赖 GDB；仅调整两条停止交接，前置 Stopped/not_produced、后置保留正文并 class aggregate Partial/Cancelled。实现后另保存最终源绑定、原始失败/成功流与新 fix commit 自身适用 CI；既有 c814/四工程结果不代替本片完成。

规划边界补充：保存的旧独立审计描述非 Complete 丢失的机制；本片实施只将其中真实承载预算/取消/期限的 request-ending stop 传播为共享请求终止。value-level Unsupported/decode 不完整依既有隔离/汇总处理；保留全局 merge_execution strongest-priority（取消高于预算），不新增全局 first-wins。
