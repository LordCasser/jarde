# Root verification — recover-proved-local-source-types

当前 **6/8**。类型生产补丁已于关闭后的 main `5c2c06f1ec8c3ff0560f2c2d89059ee7d6d06b02` 重新应用；整 CF12 未接受。独立 pop 来源片已6/6。新增永久真实 class 测试实际 **9/0/0**；14条完整本地验证已实际通过，645/0/2；新CLI已冻结。完整源码对照已接受；自身 CI 与最终主线交付待完成。

## 第一次应用与架构边界

两轮真实 focused 测试均为 2/1：null/String 全物理来源及公开 atomic Stop 通过；char 正文正确，但缺 append 后 pop82/92/105。实际 patch、永久测试和失败 raw 已保存，随后恢复生产源码。不能将这次失败记为任务2.1/2.2/2.3通过。

类型恢复只扩展既有局部声明类型决策：准确 char producer 种子加全部写的有限兼容证明；null 首写且所有非 null 写为同一准确 Reference。copy/phi、混合写与槽复用保持原边界。root 已完整阅读 saved typed patch/test 及周边声明、调用和选择器逻辑；现有 invocation_argument 按准确 descriptor 转换 primitive，足以保留原 append(I) 重载，无需新重载机制，仍须完整类重编运行及新 javap 证明。

预算 observer 私有稿引入四个 test-only 实体、thread-local 与生产 cfg 分支，root 审查后未应用。后续用临时真实 trace 取得准确证明前缀，保存后移除；永久测试沿用真实 IR 与公开 Stop/API，不保留观察框架。

## 重新应用与真实边界

[focused-root-v3](results/focused-root-v3/execution.json) 实际3/0/0，char 锚 pop82/92/105 全部具有准确物理来源。随后临时追踪真实恢复调用，保存 [proof-trace-root-v1](results/proof-trace-root-v1/execution.json) 两项实际测试与全部原始输出。CHAR 的 write29 前 AnalysisSteps=321，下一次 charge=1；NULL 首写1 的 bulk proof 前=89、uses=6。临时 trace 已按 pretrace SHA 恢复产品；实际临时 diff 和恢复 SHA 单独保留，无永久 observer、thread-local 或 cfg 分支。

Luna 私有测试稿经 root 修正三点后应用：补齐 helper 中漏掉的全物理来源断言；预算必须绑定测出的同一 CHAR/NULL class，不能误用其他 boundary method；no-debug 槽冲突的 number 真实名为 local1。最终 [focused-root-v4](results/focused-root-v4/execution.json) 实际9/0/0。两份 debug/no-debug class 中4类 char 种子、3类 int 反例、同型 String/null 和3类宽 Object 反例共22个方法profile均 Structured、全部物理 BCI 覆盖、default/all正文及map相同。

possibleSlotReuse 两版保留准确 Fallback：同一 local2 的 write10=int、write17=Object；BCI61 读取未声明局部被拒绝。缺失物理来源准确为 `[0,1,4,6,9,16,22,28,31,32,34]`，永久测试约束这一现有失败，不声称已解决拆槽或整个边界类已可运行。两条证明内预算以 limit321/94 分别实际 Stop@29/1，维度AnalysisSteps、written0、无content/binding/text/map。公开预取消测试仅证明恢复边界原子性，不能当成 token 在证明内部被设置的证据；新增循环的 charge/poll 静态核读已完成。

## 已独立接受的 post-pop / pretyped 基线

使用冻结 pop CLI SHA251d3d4e77773a6783df6a10564ad8cf82e67f1405ccd424558f1a0ee5e6bcde，在没有类型改动时实跑 **48命令/6用例/8输入class实例/16整类报告/38方法profile**。root 独立 verifier 实际退出0；仅九处已证 pop 成为完整 Stmt derived，删除这些准确新增后，旧 segments、primary、span、方法身份及完整正文逐字相同。原程序 fresh stdout/stderr 与历史 original/JADX raw 相同，旧 Jarde 失败分类从 raw 重新核对。全部输入、class、SDK/helper、工具和原始输出重新核 SHA。资源差异仅允许非递减 AnalysisSteps 与 elapsed，其他计数保持。

完整上游控制矩阵为 pinned JDK23、Java8 source/target。第一轮 JDK8 在 ORIGINAL FallThrough.check 加载原 JADX assertion helper 时失败：helper class55，JDK8仅接受52；18条已执行命令和原始异常均保存。未改 helper、去掉 check 或采用 stub。两份自包含类型锚与独立边界可以另行双 JDK8/23 验证，不能声称全部上游控制已在 JDK8 运行。

接受记录 [pretyped-baseline-root-v1/acceptance.json](results/pretyped-baseline-root-v1/acceptance.json) SHA **bc9be073af6fd98817d1bae6e541f630efea714cc2856a3981ff2bde76c7df06**；execution SHA **93dc97cf213358e94d56bfe52846db79ffe765a9fd187c5e5bad3ed56433bc08**。接受记录的绝对路径仍指原保留目录 `/private/tmp/jarde-cf12-post-pop-baseline-root-v4`，历史路径/hash不回写。235份原字节归档均实际核同原，copy manifest SHA00170c1ee4b6d5975bb76f945294f24f9c6708ba9cf4cf340c6c356fa0aae6e3；新增解释 README 单列。原始 char mismatch、NoDefault/conditional 编译失败仍保留，不能由这份基线接受类型产品。

## 接下来的验收

类型 patch 已按保存字节应用，未覆盖 pop 修复。保留两锚全部 physical BCI，尤其 TestSwitch 原 fallback 未覆盖的 pop105；其他控制与本次独立 post-pop 基线对照。实际核一般 int、范围外 literal、混合/未知引用、copy/phi/复用与准确 append(I) 重载边界，失败记录准确层级。完成 proof 内预算 Stop、完整类重编运行、新 CLI 冻结、自身产品 CI 与全 worktrees clean 后才能关闭本片。

root 已全文读 replay v2、v3，均有静态缺陷，未执行候选。v3仍有生成class相对路径口径不一致、raw数组与item对象比较、javap switch表数字误作opcode BCI及未准确处理既有边界拒绝等问题；Luna保留旧稿修到v5，root全文审查v3与全部后续diff，准确修raw/schema/class-set/runtime及失败profile赋值问题；[review-root-v2](results/replay-draft-review-root-v2.md)和私有稿均归档。该私有稿后来经root实际执行并修正输入schema及guard globals绑定；执行失败未覆盖，实际接受见下文。

[local-validation-acceptance-root-v1](results/local-validation-acceptance-root-v1.json) 重新核28条raw及精确测试摘要；第一次 p3_patterns 为84/1/0，准确失败是旧成本61新增两个已计费局部写后变63。按该原class的 istore_0@3/astore_1@9 更新成本与说明、保留tight正文比较，生产无修订。原失败v1未覆盖，重跑v2全部14cmd实际通过：645/0/2，target峰值701852711bytes。CLI e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403，0555；build SHA18a9750cdeb0d8bc45786f5d0766cbb256f1c8a87b1daf350965e910535a6413。

## 完整源码实际接受

[complete-source-root-v4](results/complete-source-root-v4/execution.json) 实际99条命令，由 [root verifier v2](results/verify-local-source-types-replay-root-v2.py) 独立接受：两锚 Java8/23 × default/all 共8条 runtime 与真实 original/JADX 完全同 stdout/stderr/exit；两锚全部物理BCI、10组CF12 default/all body/map对照通过，其他8个控制profile及非目标方法的正文/map/质量/拒绝保持独立post-pop基线。全部class/check/Inner与真实SDK保持。整CF12仍未接受：条件fallthrough编译失败和Labels数字投影未由本片解决。实际verifier invocation在results/complete-source-verifier-invocation-root-v5，返回0；完整raw在该片专属目录。

原边界类两JDK source运行通过，候选完整类因准确slot-conflict fallback缺return而无法编译，未进入runtime。初次JADX边界编译还含Runner包名错配；[boundary-jadx-package-correction-root-v1](results/boundary-jadx-package-correction-root-v1/execution.json) 仅给原Runner添加生成类相同package、完整JADX source原字节不变，两JDK仍唯一报107行 `System.out.println(z ? null : null)` 的 println(char[])/println(String) 重载歧义。该观察准确登记JADX缺陷；Jarde的Object局部和准确println(Object)转换在永久方法测试中保留，不能从组合类失败声称完整运行通过。

准确int重载 [int-overload-root-v1](results/int-overload-root-v1/acceptance.json) 实际9命令独立通过：char producer与46常量局部恢复后，两profile完整类编译/运行仍输出46\n，fresh javap准确append(I)、无append(C)，完整物理来源和default/all body/map相同。private verifier误将原符号工具路径/relative classpath当成resolved absolute，root按原record及SHA修正路径口径；第二次仅javap file-date中文在LC_ALL=C中变问号，root核raw diff仅该一行，其他每行/class SHA均相同。两次失败raw保留，最终root v2 verifier返回0。

收集器两处独立问题与一次root执行错误都已保留：原boundary tools keys实际javac8/javac23；runpy返回mapping不是执行函数globals，OUT未生效，root中止后将146条raw按byte-exact relocation manifest归档；另一次cargo clean与guard target扫描并行导致FileNotFoundError，未接受，顺序纠正后全矩阵重跑。没有修改冻结产品来通过这些脚本检查。cargo clean实际3042files/669.3MiB，target已移除。
