# Root verification — recover-proved-local-source-types

当前 **2/8**。类型生产补丁尚未重新应用；整 CF12 未接受。独立 pop 来源片已6/6，准确自身CI及35cdf3cb8干净主线交付均已实际接受，下一步于关闭后的当前main应用已保存的类型补丁。

## 第一次应用与架构边界

两轮真实 focused 测试均为 2/1：null/String 全物理来源及公开 atomic Stop 通过；char 正文正确，但缺 append 后 pop82/92/105。实际 patch、永久测试和失败 raw 已保存，随后恢复生产源码。不能将这次失败记为任务2.1/2.2/2.3通过。

类型恢复只扩展既有局部声明类型决策：准确 char producer 种子加全部写的有限兼容证明；null 首写且所有非 null 写为同一准确 Reference。copy/phi、混合写与槽复用保持原边界。root 已完整阅读 saved typed patch/test 及周边声明、调用和选择器逻辑；现有 invocation_argument 按准确 descriptor 转换 primitive，足以保留原 append(I) 重载，无需新重载机制，仍须完整类重编运行及新 javap 证明。

预算 observer 私有稿引入四个 test-only 实体、thread-local 与生产 cfg 分支，root 审查后未应用。后续用临时真实 trace 取得准确证明前缀，保存后移除；永久测试沿用真实 IR 与公开 Stop/API，不保留观察框架。

## 已独立接受的 post-pop / pretyped 基线

使用冻结 pop CLI SHA251d3d4e77773a6783df6a10564ad8cf82e67f1405ccd424558f1a0ee5e6bcde，在没有类型改动时实跑 **48命令/6用例/8输入class实例/16整类报告/38方法profile**。root 独立 verifier 实际退出0；仅九处已证 pop 成为完整 Stmt derived，删除这些准确新增后，旧 segments、primary、span、方法身份及完整正文逐字相同。原程序 fresh stdout/stderr 与历史 original/JADX raw 相同，旧 Jarde 失败分类从 raw 重新核对。全部输入、class、SDK/helper、工具和原始输出重新核 SHA。资源差异仅允许非递减 AnalysisSteps 与 elapsed，其他计数保持。

完整上游控制矩阵为 pinned JDK23、Java8 source/target。第一轮 JDK8 在 ORIGINAL FallThrough.check 加载原 JADX assertion helper 时失败：helper class55，JDK8仅接受52；18条已执行命令和原始异常均保存。未改 helper、去掉 check 或采用 stub。两份自包含类型锚与独立边界可以另行双 JDK8/23 验证，不能声称全部上游控制已在 JDK8 运行。

接受记录 [pretyped-baseline-root-v1/acceptance.json](results/pretyped-baseline-root-v1/acceptance.json) SHA **bc9be073af6fd98817d1bae6e541f630efea714cc2856a3981ff2bde76c7df06**；execution SHA **93dc97cf213358e94d56bfe52846db79ffe765a9fd187c5e5bad3ed56433bc08**。接受记录的绝对路径仍指原保留目录 `/private/tmp/jarde-cf12-post-pop-baseline-root-v4`，历史路径/hash不回写。235份原字节归档均实际核同原，copy manifest SHA00170c1ee4b6d5975bb76f945294f24f9c6708ba9cf4cf340c6c356fa0aae6e3；新增解释 README 单列。原始 char mismatch、NoDefault/conditional 编译失败仍保留，不能由这份基线接受类型产品。

## 接下来的验收

重新应用 saved typed patch 于关闭后的当前 main，不能恢复历史 HEAD 覆盖 pop 修复。保留两锚全部 physical BCI，尤其 TestSwitch 原 fallback 未覆盖的 pop105；其他控制与本次独立 post-pop 基线对照。实际核一般 int、范围外 literal、混合/未知引用、copy/phi/复用与准确 append(I) 重载边界，失败记录准确层级。完成 proof 内预算 Stop、完整类重编运行、新 CLI 冻结、自身产品 CI 与全 worktrees clean 后才能关闭本片。

root 已全文读 replay v2、v3，均有静态缺陷，未执行候选。v3仍有生成class相对路径口径不一致、raw数组与item对象比较、javap switch表数字误作opcode BCI及未准确处理既有边界拒绝等问题；Luna保留旧稿修到v5，root全文审查v3与全部后续diff，准确修raw/schema/class-set/runtime及失败profile赋值问题；[review-root-v2](results/replay-draft-review-root-v2.md)和私有稿均归档。v5尚未候选执行，不能把脚本静态完成算实际通过。
