# Indexed List 参数声明：当前证据入口

当前 tasks **5/6**：Luna v2/v3实现已集成，root focused、适用广门、最终完整源码与双VM对照均接受。产品提交、自身CI及fresh workspace尚待，不提前关闭。最终CLI SHA `90b30ad1132bdc904c20f1b4624347b84d37637c78f1f3e291ce2f0dea91f64c`，63809216B；基线仍为096a764dc/CLI56a589ef。

- `planning-and-dispatch-root-v1/dispatch.json`：实际Luna私有实现派发、作用域、共享写权限与集成前置条件。
- `planning-and-dispatch-root-v1/root-implementation-review-decisions.json`：root及独立对抗审查，v1实现退回的具体修正；同目录保留原审和撤回的嵌套类型假问题。
- `primary-baseline-root-v1/capture-review.json`：273归档成员全部root逐字复开。完整上游原类4VM/40case一致；Jarde16次完整重编/32VM/320case同原，全部8API物理方法/Code/BCI/UTF8 map及default/all平面闭合。
- `controls-baseline-root-v1/capture-review.json`：439成员全部root逐字复开，含两版完整原始控制、全部采集/原始流/真实失败与被替代checker；最终独立checker实际exit0。

JADX1.5.6在完整固定上游的no-debug输出将get移进inner while条件。其JDK23重编后的两VM各有4case出现重复get事件；本组返回/异常相同，不能称语义一致。两次完整JADX outer源的JDK8重编为真实major55测试框架边界。

控制v1的显式null-throw循环有独立的uncovered block27/throw36整方法拒绝；16次Jarde完整重编均因拒绝占位符失败，无Jarde VM。v2是只将原始void probe换成value.length()的新构造源，保留String cast和异常；其16次Jarde完整重编/32VM/512case及16wrong-root全部接受。两版本JADX各4次完整重编因String→Integer直接转换非法失败，零JADX VM。每版本原始JDK23 debug/none各双VM64case，加实际JDK8原类双VM32case同原，未借用原target运行生成类。

真实源码/class已由root从v2原始编译输出冻结到tests/fixtures/java8/indexed-list-parameters；corpus fingerprint经显式受控重建与正常校验，2435资产。主java8-recovery已同步一requirement/五scenario，18/18规范校验通过；固定账本仅记录本片范围，不增加CF11或71/612完成数。

## 实现与最终完整对照，5/6

`implementation-and-validation-root-v1`保留Luna v2/v3原补丁、root审查、所有真实失败/清理和最后有效检查。root修正测试字段/导入、移除单项公共adapter及无get空walk；实际正向失败定位旧census固定Stack(0)漏size receiver，Luna v3按实例opcode/kind、descriptor消费值数量、唯一最低实际stack depth闭合receiver，参数实参不混作接收者。root补descriptor/set分配前计费、集合工作上界及逐行poll。无新IR/pass/目标事实记录。

最终focused40通过/0失败/1旧忽略；此前指纹6通过/1旧忽略、batch2通过。适用广门578通过/0失败，fmt/CI Clippy/MSRV1.88均通过，3198来源相同。broad v1的548测试全过后，旧清理器只接受一个二进制，因本轮四target命令而停止；保留这次真实harness失败，v2仅续跑其余30测试和CLI构建，不重复已过检查。所有Rust exactclean且target absent。

`final-complete-comparison-root-v1`：最终CLI实际184采集命令/368raw，完整上游及完整独立控制v2共40次生成源码编译，34成功/6已分类失败。Jarde32次整类编译全部成功，64VM/832case全部同原，双编译器/双VM/全部Code及BCI/隔离CodeSource与wrong-root均核准；原始上游4VM40case、控制4VM64case同原。JADX两次已知no-debug事件差异、两次JDK8 major55框架编译失败及四次非法String→Integer编译失败保留，失败不运行。

独立API差分接受16 selectors：五个正向方法共40cells恢复精确List<String>/原结果及精确成功marker，真实cast和原正文不变；两个负向方法仅16cells的拒绝说明从旧body缺证变为既有统一return-source缺证，拒绝类别/擦除声明/正文均不变，其余负向全部原样。所有raw outcome/report/map除usage外逐字一致；wholeclass只允许上述确切变动。32个成功Jarde生成class的精确Signature与String/Integer物理cast再次核准。

独立checker的helper缺参、旧template遗漏、success marker原来在method内部而refusal在header外等准备/实际错误均保留；root仅改匹配真实schema/精确标记位置，未改生成源码、driver、oracle或放宽raw/map一致性。最终control checker root-v1、API delta root-v2及原始Java/class/driver/JDK/oracle binder实际exit0。三个新归档666成员全部root逐字复开；含两个基线共1378成员。

`final-acceptance-preparation-root-v1`仅为下一阶段native/CI准备，不称已运行。保留14源码/9fixture库存，所有旧门+七个新门，官方ZIP须每step独立成员齐全，不能aggregate拼造。
