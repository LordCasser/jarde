# Tasks

## 1. 实施基线与永久反例

- [x] 1.1 在reference片自身CI/归档后重绑定实际实施head、两FO原编译腿与未改原Java/driver/oracle；冻结永久fixture并同步语料指纹。验收：Code/声明/输入SHA、原四次1/100与四次60B控制、完整旧FO重编exit1及具体bump拒绝、所有raw可移交；只新增预期fixture指纹条目，不删样本或借历史CI。
- [x] 1.2 补入具名库/CLI与最窄证明期回归，在旧生产先证实行为红。验收：旧红因静态prefix新值缺口而非路径/工具缺失；校物理字段读写、dup新值消费者与来源，预算/取消必须在实际证明工作中观察，不以入口预取消代替；逐项保留未执行/失败边界。

## 2. 局部恢复与兼容性

- [ ] 2.1 实现有预算的静态int新值返回证明、原子认领和最小前置更新AST/发射，复用字段/Return/来源接缝。验收：同物理字段与可更新声明、sum→dup→写/返及唯一consumer全部闭合；错owner/name/type/staticness、旧值、非one/add、alias/效果/handler/跨块等控制不被本规则误认；真实预算/取消不提交部分证书或产物，原原因与usage保持；同步边界文档。
- [ ] 2.2 核既有实例字段pre/post、static postfix旧值、void字段单位更新、compound/数组/条件位置与FO已正确finally。验收：选择受影响定向用例实际通过，正常文本/来源按各自契约保持；其它规则可正确恢复的负控制不强迫全局拒绝，不扩全局dup求解或改finally规则。

## 3. 完整源码与产品门

- [ ] 3.1 对最终冻结CLI两FO腿的全部report.text原样重编并用隔离rebuilt-only classpath在Java8/23跑固定main及未改独立driver。验收：四次rebuilt main均1/100、四次driver均60B六行exact且非零/溢出/return-before-finally吻合；原FO/driver/oracle与生成文本不改、不删方法，物理读0/常量3/add4/dup5/write6/return9来源与实际方法绑定，raw/argv/exit/输入与源码before-after齐全；失败保留，原类成功不替代Jarde重建。
- [ ] 3.2 完成适用格式、CI参数Clippy、语料指纹、两版OpenSpec strict及完整workspace/MSRV/workflow与该产品commit自身CI独审。验收：所有命令实际非空、source/CLI/CI head绑定、默认stack与5GiB free/1GiB target磁盘门、exact clean后无target；旧失败/skipped与未覆盖范围保留，不借前片或旧head的CI通过本任务。

## Workflow follow-up

- 当前2/6；1.1与1.2完成，规划与原类控制不算恢复实现。reference片的82611166自身CI已通过、正常归档并交付main fc195175；本片从该实际head开始，不能借前片CI完成本片产品门。
- 实际验收后同步本片delta至主spec、正常archive并核实主线源码/证据绑定；不增加全局CF11/CF18/71/612覆盖主张。

## 实施进展（2026-10-16）

- 原FO/driver/oracle与两份790B class逐字冻结，field与Code物理身份核实。新16命令基线中，两套原FO重新编译逐字相同；四次main输出6B、四次driver输出60B在Java8/23全部exact。两次CLI全部5方法的report.text原样保存，完整javac均退出1，missing return在bump()I；evalThenFin的保存值/finally原已正确，rebuilt runtime为0。
- 显式再生成实际1通过/0失败，语料2220→2226恰新增六个非md输入，旧条目及所有分类字段原样。首次采集因regenerator在测试名称与ok之间打印状态而解析失败，原raw与clean保留；后续正常指纹套件5通过/0失败/1默认忽略，没有伪称首次后继测试已运行。
- 旧生产公共测试1通过/2行为失败，JSON CLI单项0通过/1行为失败，具体为未呈现新值返回和字段read/write均false。两红及CLI均在第一个javac23腿的首断言退出，第二javac8腿和后续来源断言未到；既有finally顺序控制实际两腿通过。双腿完整源码红另由16命令基线证明，不能互相冒称覆盖。
- 新旧生产CLI从实际native构建冻结，59,657,968B、0555、SHA256 c44bde2cdbbf32f70a29bef345c19f11b86229b2e399a0936ed90c1a264fa396，与此前旧CLI逐字相同；Cargo artifact的fresh实际true（此前CLI测试已构建），不称新实现或fresh=false。exact clean后target缺失。
- 基线完整源清单4914=原4907加7fixture文件；旧native收据实际selected4902，未纳入13个fuzz种子，不能将其改称完整4915。正常指纹门实际验证全2226，独审另读13种子与基线相同；两个范围分别记录。实际source/inputs before-after与raw见results/old-baseline-v1。
- 额外两条受控javac命令确认同名字段可遮蔽owner：合法手写原控制可编译，独立手写的++PrefixOwnerShadow.i候选报int cannot be dereferenced。此处不是Jarde生成结果，也没有改原FO/driver；实现须保护合法owner拼写，不扩全局名称解析。
- 首轮新实现编译成功，但把dup误判成Other导致两证明测试失败；修为项目实际Duplicate后，证明与取消测试通过。第二轮公共库2通过/1来源断言失败：现有source-map API把owner路径与Field子节点都按primary BCI检索；初始count=1错误。保留失败，改为精确断言两文本FO/FO.i及Derived/Direct provenance，并增加全部origin的实际方法身份检查，未弱化来源约束。
- fixed-native-v3实际10命令：proof3/0/0、public3/0/0、CLI1/0/0、void单位更新3/0/0、postfix4/0/2、compound7/0/0、条件postfix4/0/1、条件compound4/0/1，合计29通过/0失败/4默认忽略。三种独立内存class变体经实际分析获得SSA后验证非one/subtract/old-value不被新proof认领；原fixture文件未改。真实late IrItems拒付和最后accepted charge之后取消均无证书，原StopReason与已接受usage保留，未声称动态验证deadline。
- 本片新CLI已冻结为59,699,272B、0555，SHA256 3596a781705c46e51a182d45cd7a54ea8f448cf6951e1f4ce3f07cf5a00af821；本次完整union4915项source前后相同，exact clean后无target。1.2完成，2.x/3.x仍待剩余边界、完整源码与产品门验收。
