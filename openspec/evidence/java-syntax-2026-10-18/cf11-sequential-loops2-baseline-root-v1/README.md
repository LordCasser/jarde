# 固定 TestSequentialLoops2 完整原始基线

2026-10-18，root 实际采集并独立接受24命令/48 raw。上游完整 `TestSequentialLoops2.java` SHA256 `adb34f80f7d55cd3309ed7708f0fdb46768cf32e4004dbd13d4e278530306720`，保留外层 JUnit 类和全部子类。javac23 `--release 8` 的 debug/none 两份输入均为 major52；每份包含 outer/child 两类、各3个 Code。child 的构造器/ascii/clinit 物理 BCI 数为3/57/8，两 profile 的完整 Code 字节相同。javac8 编译完整原始源真实遇 IntegrationTest major55/expected52，保留错误及零 class 产物，不裁掉外层绕过。

冻结产品 `ee013adb35711366d0aff2f774ee633b34c06082`，CLI SHA256 `aabf62fe373876d176e6001bf3ed1fd84f80d2169ff5e787c4095c5d253c63d9`。8请求覆盖 debug/none、child/bundle-child、default/all；三个方法均 Structured、无 fallback，class Complete。四组 default/all 完整正文、方法文本/quality/fallback/map 相同，全部物理 BCI 与 UTF8 区间接受。child/bundle 的 BLAKE3 owner 身份一致；本次未独立重算 BLAKE3，输入物理字节另由 SHA256 绑定。

原始 driver 实际8运行/64case。每个 case 使用 fresh parentless loader，核独立 expected CodeSource、原始 `lowercases == {'a'}`，记录 UTF16、同一对象及异常类型。default 的原始短表和 configured128 的反射注入控制分开；后者不是修改原源码。null、空串、Unicode/代理对、NUL、单/多字符覆盖。原始四个 profile/VM 组合按模式稳定一致，异常消息保留 raw，不跨 JDK 硬比。一个真实 wrong CodeSource 控制 exit1。运行调用静态转换，构造器只检查存在，不声称执行了构造器。

JADX1.5.6 两份实际完整 bundle 输出保留。它使用临时旧值和 `while(true)`；Jarde 保留两个 `while(i-- > 0)`。黄金源码断言并不要求字面 i--；本次审阅断言但未执行上游 JUnit。文本形状不代替[生成源码重编运行验收](../cf11-sequential-loops2-generated-matrix-root-v2/README.md)。

receipt `execution.json` SHA256 `9811af1be7efd2d5f4ce38d7ed7ae1ce426ca5f30de9b6cac8b4344553211a37`。root 阅读并实际执行独立 checker 后接受；脚本、报告及物理 parser 在 archive extra 内。3298来源包含原构建3187文件，后者无缺失/变化，另111项是已有 exception-only 证据/工具目录；不能把两种 closure 计数不同当代码漂移。CLI/依赖/JDK/来源前后不变，5GiB/target1GiB/default stack 守卫接受，target不存在。

归档91成员，SHA256 `b7473a49fcc3aae45fb83e23b80b5a43297f5d426a87608b7c072fac32e45774`。本片不增加 CF11/71/612 完成数。
