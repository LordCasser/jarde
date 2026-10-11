# CF11 TestSequentialLoops2：同IR的构造器Phi首拒

2026-10-12，root实际守卫运行两命令：准确临时测试1/0/0和cargo clean。只有init.rs被临时替换，finally恢复原字节；完整4718项源码前后/live同finalbuild-v3，runner/candidate/tool pins一致，默认Rust stack、5GiB free/target1GiB一秒守卫，peak161069529 bytes，target不存在。原始stdout/stderr见diagnostic，root独立验收见root-acceptance.json。本观察独立于for-condition产品验收。

准确冻结class为931bytes/SHA256 `524b1746bdff10a3b9da021679f9832202df1552c2eb42c02d150284b7dd8664`。选择实际static asciiToLowerCase(String)，参数slot1/accessflags0x0009。v1错方法/API、v2参数slot0是未运行私稿，保留不称失败工具链；实际只跑v3。MethodFacts第三参是参数槽数，不是flags。

实际canonical CFG13块18边，无unreachable。尾部83比较分向87/91，两臂唯一正常接续99。真实SSA：new91输出V49；dup94输出V50（unused）和V51（ctor96读取）；ctor96写V53；V53唯一use为Phi99、bci=None。Phi99 Stack0 V14准确输入为load87的V48和ctor96的V53；V14唯一Some99消费是Return99。不能把V50的unused或外部copy marker当已确认首拒。

对同一SSA实际调用outside_readers([91,94,96])返回空；Sites测试上下文记录候选91/java.lang.String verified=false、site_at_head91 absent，jre_new_shape“nothing reads instance”。init.rs的现有usage.bci()==None skip与实际SSA共同说明Phi operand读者被忽略，这是首个已观察到的Sites候选拒绝；外部原基线缺88/94/95/96和完整类编译失败仍保留。

范围限于同IR SSA/CFG/NormalFlow/outside_readers/Sites，Sites测试使用现有empty field/concat和ArrayInitializers上下文；没有执行Region或Builder完整conditional proof，也没有重新运行class-source。因此不能声称后续所有门槛均通过。两循环已经在原三方基线正确恢复，缺口属于构造器条件值（CF04/EM05交界），不扩CF11 loop机制、不增加71/612完成数。

下一架构判断：能否在既有Site所有权门复用“finished constructor唯一Phi operand→唯一physical consumer”，再由Builder保持完整conditional CFG/Phi/consumer证明；前置新CFG证书是候选方案，尚未证明必要。必须先核已有Site发布/消费及失败路径和对抗边界，再形成最小OpenSpec并派私有实现。不把宽泛Phi traversal/任意alias准入当最小修复。
