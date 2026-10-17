## Context

动机见 proposal.md。当前生产基线为 3fa506df0，最终 CLI SHA7287e856…；本地 main ec59b498a 仅后续文档。原始 TestFinally3 源码 SHA0671606e…，来自 JADX checkout2fb1b163…，两份新输入由 javac23 `-source 1.8 -target 1.8` 加 `-g`/`-g:none` 生成；不冒称 javac8 编译了 major55 的测试 harness。全部七方法 Code 与异常表相同。JADX1.5.6 为独立比较器，其不正确输出不作 oracle。

真实临时 SSA 探针已执行，源码恢复、exact clean：getInputStream 的数组 BCI5 写 ValueId5，唯一读取为构造 BCI7 的 Stack2；物理 receiver 为 Stack1/ValueId3，参数依赖仅 {4,5}，无 handler，无数组初始化证书（零长不需要 stores）。两 profile 首拒均为 jre_new_interleaved_effect。探针 raw 与完整类基线先放 root 私有目录，交付时归入 results；不以临时探针构建来源冒称产品。

ArrayInitializers 已证明非空初始化的最终值、唯一 consumer、完整 sources、元素/child 和 pending construction sites。Builder 已渲染裸 newarray/anewarray/multianewarray 的长度及初始化表达式。现有 inline_argument_chain 只允许 consumer 严格位于 dup..ctor 内；普通构造也只接受 argument_dependencies 中的中间 Invoke。该组合接缝是修改位置。

## Goals / Non-Goals

**Goals:** 在现有证明之间传递精确最终值/消费者身份；共享计费；复用既有表达式、异常覆盖与原子 materialization。不引入 new solver、通用 alias 机制或独立数组重建器。

**Non-Goals:** 见 proposal。不能把数组所有 stores 加入普通参数依赖集合来绕过 effects 判据。不能删除 TestFinally3 失败方法或用转写 Tf3/插桩 probe 替代原始类验收。

## Decisions

1. 保持 value_dependency_bcis 的定义，区分“调用产生参数值”和“构造器直接读取数组最终值”。前者复用原有路径；后者必须匹配物理 operands 中跳过 receiver 的准确值，并核唯一使用的 block/BCI。不得仅令 consumer==ctor 即接受。
2. 裸分配使用其真实输出及唯一构造使用，长度仍在既有依赖/effects 闭合中验证；正长度和多维无需新 stores 证明。初始化数组只读取现有完整证书，保持全部 sources 严格位于外层参数区间（consumer 可等于 ctor），保留 element/child/pending site 所有权。避免复制 proof plan 或重复拥有 site；真实组合 probe 覆盖 constructed elements 和 nested children。
3. 将接受的完整数组来源交回现有 inline_arrays 与单次原子、statement-free、全构造至唯一消费点 handler coverage 检查。Builder 使用原有数组渲染，完整保存 length/store/element/ctor 来源；如果现有 Builder 在真实案例仍首拒，先查清原因再调整本 design，不靠缩小断言凑绿。
4. 不增加库依赖。参考 JADX 已有源码、测试和算法定位问题；本片复用自有现成数组/构造设施，无须复制 JADX 代码，也无新增第三方许可负担。
5. 统一普通构造准入，移除被通用完整证明覆盖的旧 String(char[]) 窄证书与相应常量/owner/直接 return 限制。它读取同一 ArrayInitializer，只额外缩小条件；真实 effectful String probe 已有完整数组候选，不存在独立机制需求。若实现发现旧证书存在通用证明不能表达的真实能力，须列出实际事实后再决定保留，不能增加无用 fallback。原 wrongOwner/wrongDescriptor/release 白名单测试改核真实语义，额外 use、错误值与 handler 边界继续拒绝。
6. 首轮真实库测试457通过/2失败暴露 reader 的额外闭合缺口：String([CII)V 和两个数组参数中的第一数组尚无完整候选，不能声称它们原已有证书。现有 array_initializer_reader 在最后 store 后扫描最终调用的全部其他 operands，把早已产生的 receiver/前置实参也要求在此窗口重建。针对真实构造消费，忽略窗口之前已产生的 operands；窗口内后置参数仍用既有 expression 闭合或已完整证明的 sibling array 事实覆盖，禁止无关指令/外部使用/效果漏入。借用已有 ChildArrayFacts，不把 sibling 的 sources/owned 变成当前数组所有权，不全局放宽所有调用。遇到新的实际闭合问题先记录事实，不将已承诺多实参正例降为拒绝。
7. 同 owner 重载的真实字节码确认旧扫描误选元素的 ctor15 为外层 head0 的构造调用。应在选中调用时用已有 SSA receiver 身份匹配当前 allocation，错 receiver 继续扫描；选中后仍做原身份、参数、effects、异常和唯一消费检查。不得以 owner 名称相同推断对象相同，也无须引入新控制流机制。
8. 完整工作区回归实际暴露重复工作计费：选中 receiver 时已经读取 operands，后面不再读取第二遍；物理参数次数在既有已计费 producer 检查循环中统计，不再另外扫描一次。保留选中后的 receiver 身份复核及所有参数证明，不调整 P5 旧表值、预算或断言。修正后的全部完整源码及非 usage 报告须与已验收矩阵一致；实测仅允许 ir_items 与 elapsed_millis 数值变化，不能凭递归忽略整个 usage 子树放过其他差异。

## Risks / Trade-offs

- [同一调用读取多个值导致误认数组] → 判断精确最终 ValueId/物理 argument 位置和单一外部 use，增加错误身份、重复读取控制。
- [初始化链和构造元素重复拥有或执行] → 根审查真实 SSA、pending sites、expression/owned，完整类运行核 identity、次数及次序。
- [长度/元素抛异常移动] → 保留依赖执行点与整个构造区间/唯一消费点的有序 handler 集，核负长度、生产者异常、构造器异常和覆盖变化拒绝。
- [便利性扩大测试结论] → 所有方法保留；编译失败记 no-runtime；原源码、两编译器、两 JVM、两 evidence、child/bundle-child/owner 各自记账。owner 因 harness major55 只 javac23，成功生成的 target 可双 JVM 隔离运行，不执行 JUnit owner。
- [多个实参被字节码重排] → 用验证器合法的 swap 变体核未被证明的栈重排仍被 effects 门拒绝；正常 javac 的递增 producer BCI 测试本身不能证明所有输入次序安全。

## Migration Plan

先冻结真实旧基线和通用数组探针，再由 Luna 交付最小私有补丁，root 审查并串行集成。运行针对性/公共停止/来源测试及完整双编译器对照，再跑当前完整 workspace 和产品 CI。构建始终默认栈、5GiB 可用余量、target 1GiB 上限、每秒 process-group 守卫及最终 exact clean。归档原失败及最终成功，交接区分产品/文档 head，持续 goal 保持 active。
