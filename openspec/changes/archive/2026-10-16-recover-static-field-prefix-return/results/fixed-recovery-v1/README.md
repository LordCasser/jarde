# 静态 int 前置更新：完整恢复源码实测

实际产品提交 c075d1a15ab9e29482ee2eb006bc152e5c57980c，冻结 CLI 59,699,272B、0555，SHA256 3596a781705c46e51a182d45cd7a54ea8f448cf6951e1f4ce3f07cf5a00af821。4915 项完整源码/指纹输入前后相同，逐项 Git blob 与该提交一致。CLI 二进制外 pin；包内保留实际构建收据和源码绑定，不另造执行。

两种原编译器 FO.class 经该 CLI 输出的完整 report.text 均为 1617B，SHA256 eabbf22187e0b3c24bb984d9496a4785acc479fe05ca9e07acf8952337e25811，逐字相同。旁附 FO.java 是这份原样输出，未修文本、删方法或变更原 Java/driver/oracle；两腿全部五个物理方法均有报告。bump()I 恢复为 `return ++FO.i;`，字段读 0、常量 3、加法 4、dup 5、写 6、return 9 来源保留。

真实重放共 14 条命令：两次完整 JSON CLI、两次完整生成 FO.java 编译、两次未改 driver 编译，以及两编译腿乘 Java8/23 的四 main 和四 probe。javac 的 classpath 仅该次空 rebuilt 输出目录、sourcepath 为空；driver 仅对 rebuilt FO 编译；运行 classpath 只含该次 rebuilt FO/driver，没有原 class/JAR。所有 exit=0、stderr 为空，四 main 输出精确 6B `1/100\n`，四 probe 精确 60B：

```
0/0
8/8
-2147483648/-2147483648
8/100
-2147483648/100
1/100
```

两份生成源码编译前后逐字未变，输入/源码完整清单前后相同，default stack、5GiB free/1GiB target 门保持，target 前后缺失。此前旧生产两份完整源码 javac exit1/missing return 的原基线另见 old-baseline-v1，不能借原类运行替代本轮重建。

focused-native-v3 实际 10 命令，29 通过/0 失败/4 默认忽略；三种真实 class 内存变体重做 SSA，验证非 one/subtract/旧值消费者不被新规则认领。真实 late IrItems 在 BCI9 拒付、最后一次 accepted ledger charge 后取消均不发布证书，已接受 usage 保留。四个 ignored 兼容 runtime 没有在此 focused 命令集运行；不声称新增动态 deadline 实验。 静态组合复核另核固定 c075 的 9 个 Git/source pin：所有 proof poll/charge 与 helper 的 StopReason 经 ? 传播，最后 poll 后才交证书；新 PrefixUpdate 只经既有 checked writer。这里的原子性指正文提交前的未提交正文与来源；后置可选 evidence/source-map 较晚停止遵循既有契约，可保留完整正文与明确标记的证据进度，不能泛称任何停止都清空已交付产物。

首次实现把 dup 误核为 Other，两证明失败；第二次证明通过、公共库来源测试因初始 count=1 与实际 Path/Field 两节点冲突而失败。两轮原 raw 与未执行后继完整保留。第三轮来源断言精确检查 FO/FO.i 两节点、Derived/Direct provenance 和全部物理方法身份，未减少断言。prepared replay-v1 未执行，因误指向原 FO.java 被 root 审查拒收；v2 才实际原样编译生成源码。一份未提交的静态设计审查缓存草稿曾误覆盖，原 pin/不可恢复状态另有 lineage；没有丢失执行、raw 或原/生成源码。

当前 owner 证明局限同次本类字段表无首标识符冲突、已知 Object 直接父且无接口，和 class/method header 无待解析 binder/member-type 上下文。class Signature/InnerClasses/EnclosingMethod 或当前 method Signature、未知/歧义 header 均按缺证保留原拒绝，不声称已支持无冲突泛型/成员类型。旧 instance/postfix/void/compound/finally 规则保持自身契约，不增加全局覆盖数字。

独审：focused-native 99 检查/0 错误；完整生成源码 19 检查/0 阻断。此包只接受上述有界源码恢复实测，完整 workspace/MSRV/workflow 和产品自身 CI 另门。

capture.tar.gz：7,591,281B，SHA256 09fa92f2269a7fc5a635908eac175f4d4cd8d750f2077dd65a76f2c540f43232；122 payload 加 manifest 共 123 个安全唯一普通成员，包含两个失败 native、成功 focused-native 和实际 generated-source replay 共 29 命令/58 command raw。root 已重开全部成员核 hash；独立包审计再次核验全部 123 个成员、29 命令/58 command raw 以及原失败/未执行边界通过。
