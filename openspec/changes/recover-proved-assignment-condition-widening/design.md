## Context

动机见proposal。当前产品5bd、CLI90b已完成自身CI与完整indexed List验收；文档主线c0abb3adc干净。完整上游实际4VM16case仅覆盖false，因为next恒0。原始生成对照27命令：16次Jarde整类重编均缺return失败、零JardeVM；JADX两次JDK23编译、4VM16case同原，两次JDK8框架major55失败不运行。

两profile临时诊断实际显示dup9/store10/test11的赋值目标为Int、RHS为Some(Char)/next()，被duplicate_expression的exact equality门拒绝；所有区域已经structured。诊断输出除usage外等同产品，临时源已恢复d22ee8f1c40d0a98a174e759904a6a97313b85728bb3056d4aeba18ca8704df9，不把诊断CLI当产品验收。独立最小控制原始四次编译、8VM48case已接受，8API均只拒绝test(C)Z，其余五方法完整，全部raw map闭合；控制生成源码对照仍待执行。

## Goals / Non-Goals

**Goals:** 修正既有已证赋值表达式的类型关系；原目标类型、表达式位置和物理证书均不变。

**Non-Goals:** 改声明规划或区域层、增加赋值形状、引用子类型判断、一般表达式窄化、宽栈复制、装箱拆箱或新类型推断。

## Decisions

1. 在duplicate_expression的Expression分支，保留Some(目标类型)的精确相等路径；否则仅接纳有确切presented且既有widens(presented, target)成立的情况。prove_local_assignments仍只准入Int/Reference，故新增实际类型对仅Byte/Short/Char到Int。不能从验证器I推断源Java类型。
2. 直接复用widens；meeting_position在严格预检后对Position只会返回原RHS，额外调用没有作用，而其unknown/reference/常量窄化宽松规则不适合这个证书消费者。不增加helper、IR、遍历、pass或solver，不改变原拒绝说明。
3. RHS仍只经原render_value在原测试点呈现一次，LocalAssign保持目标声明类型，RHS保留自己的源类型；direct store、derived dup及published计数完全不变。其它证书缺失和真实Stop继续由既有checkpoint/整方法拒绝闭合。
4. 复用标准库、现有Java类型表、reader/SSA/Builder和完整类比较工具，无新依赖或许可负担。JADX的do/while呈现可作对照，不需要复制其区域重写；本层已有足够的局部赋值机制。
5. 复用真实Builder fixture测试seam，在原消费者上构造类型缺证/不兼容/错value或test站点和停止边界，不为测试抽生产函数。公开回归冻结完整上游与最小控制的真实编译产物，分别比较原始/JADX/Jarde，driver-only classpath和parentless加载器校验CodeSource；完整重编失败保持失败，不裁剪方法或借原target运行。

## Risks / Trade-offs

- helper宽松语义泄漏 → 仅调用widens，保留严格Some与原相等路径；测试unknown、boolean、引用、窄化。
- 扩大转换改变求值/异常 → 不修改节点或render位置；动态控制记录位置/调用/events、自然数组越界及字符65535。原常零helper不冒充true分支覆盖。
- 测试替代真实消费者 → 复用compound_test_builder及公开完整类测试；单独widens表的镜像断言不作为本片完成证据。
- 新fixture再次漏更新库存 → root从实际class/Code/branch/handler/jsr独立算delta，再更新cfg(test)计数与语料指纹；不改生产reader。

## Migration Plan

root先接受原始完整证据与本设计/strict，再派Luna私有补丁，可与控制生成源码基线采集并行；全部新基线实际接受前不集成。root独占共享源/Git/工具链/网络，Luna私有且不跑Cargo/JDK/CLI。串行focused、相关赋值/循环/类型/停止广门、最终两编译器/双VM完整对照、fresh workspace及产品自身CI；defaultstack、5GiB余量、target1GiB每秒守卫和finally exactclean。最后同步主规范/固定账本/handoff，干净main推送，只关闭本叶子。
