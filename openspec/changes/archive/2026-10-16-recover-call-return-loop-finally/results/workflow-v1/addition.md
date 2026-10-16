### Requirement: A proved loop call return keeps its multi-call finally

系统 SHALL 恢复可证明的数组循环条件try：一臂保存int调用结果后返回，另一臂继续循环，两臂与catch-all重抛路径执行同一多调用清理。输出 MUST 保持受保护调用、值保存位置、每路径清理顺序及异常优先级，保留物理来源；缺证或真实停止遵循既有契约。成功 MUST 经过完整生成源码原样重编和隔离运行验收。

#### Scenario: The loop keeps normal and early return paths

- **WHEN** 固定方法对数组逐项检查，等于3时返回`c0(x)`，否则继续，finally输出`fin:`与当前元素，循环耗尽返回-1
- **THEN** 恢复 SHALL 对空数组返回-1且无输出，对[1,2]返回-1且依次清理1、2，对[3]及[1,3]返回3000且分别清理3或1、3；MUST NOT 丢失提前返回、循环后继或把每轮清理移出循环

#### Scenario: A call result is captured before cleanup

- **WHEN** try内具有may-throw效果的调用正常完成，其int结果在清理前保存并在清理后返回
- **THEN** 调用、保存与返回 SHALL 保持同一值链和原求值位置，不能重执行调用、在清理后改读另一个值或把调用移出try；固定纯c0的运行结果不能单独证明其调用次数和时点

#### Scenario: All cleanup operations keep their order

- **WHEN** 三份清理均按System.out读取、StringBuilder分配/初始化、字符串与int追加、toString、println的同一实际依赖链执行
- **THEN** 折叠后的finally SHALL 保持每份自身的对象生命周期、输入值、调用目标与求值/异常顺序；副本差异、错误未初始化alias或额外消费者不得被本规则认领，不能只比较显示调用名与opcode数量

#### Scenario: Cleanup exceptions replace a pending return or continuation

- **WHEN** 未改driver在返回路径或继续循环路径的原println调用抛专用清理异常
- **THEN** 原类与完整重建类 SHALL 都传播该异常，只清理至发生异常的那一轮，不返回3000、不继续后继轮次，也不由catch-all再次捕获清理而重复执行

#### Scenario: Exception scopes and pretry effects remain distinct

- **WHEN** 原始受保护调用有catch-all边，而数组读取在try前、三份清理及重抛均不在保护区内
- **THEN** 恢复 SHALL 保留相同异常边界与原异常保存/重抛身份，null输入在数组取长度处失败且不执行清理；不得因清理相似而扩大保护范围或把可能抛异常的try体当作不抛异常。固定driver未触达的受保护异常路径须由真实IR证明，不伪称已运行覆盖

#### Scenario: Origins cover every folded physical path

- **WHEN** 公共库或JSON CLI成功恢复固定方法
- **THEN** 正文及source map SHALL 保留同一实际方法全部物理BCI的来源，涵盖调用/保存/return、三份清理的构造器与复制、异常保存/重抛、循环尾与退出；不得通过生成一个看似相同的finally而丢弃未证明的路径

#### Scenario: Real proof stops publish no partial ownership

- **WHEN** 候选已进入实际证明工作，预算、取消或期限在证明或正文提交前被观察到
- **THEN** 系统 MUST 保留原停止原因、位置与已准许用量，不发布部分证书、ownership或未提交正文/source map，不把停止当缺证继续；入口预取消、伪造错误或扩大stack不能替代真实证明期验证

#### Scenario: Whole source execution and parent gaps stay explicit

- **WHEN** 真实javac8与javac23 --release8原独立类经最终产品CLI恢复并原样重编，未改driver仅针对rebuilt类编译后在Java8/23运行
- **THEN** 四次217B固定输出 SHALL 逐字等于原oracle，classpath不得混原class/JAR，不改生成文本或删方法；另存完整FX/FY/FO的实际源码重编及可运行结果，剩余失败不得被本片成功或结构平面掩盖，本产品自身CI须绑定实际产品提交
