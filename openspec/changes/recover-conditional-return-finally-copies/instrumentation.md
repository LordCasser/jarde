# 旧生产的真实 Facts/SSA 与首拒取证

本记录接受的是旧生产拒绝与真实边界，不是新恢复能力。运行见缓存 `conditional-return-finally-old-proof-probe-v1/execution.json`：一个真实双腿私有测试、CLI构建、exact clean，共3命令全部exit0；4922源码前后相同。新增探针仅在cfg(test)模块，沿真实analyze_method_ir取得canonical/SSA/Code，不伪造分析表或给生产guard加日志。

## 首拒条件

两腿唯一catch-all行均为[0,16)→37。旧 `prove_finally_copy` 的handler前线性后缀实际得到normal_return=34（goto）、return_load=31（println invoke）、候选normal_cleanup=[25,26,29]，exceptional=[37,38,41,43,46,47]。前述片段非空，所有这些候选cleanup BCI均在保护区外；旧算法的下一个row.end与cleanup起点相等条件实际为16!=25，因而拒绝，返回None。探针从同次Facts重演这项判定并断言旧proof仍None，不能把这条形态强塞进原单出口直线证明。

这里只固定本方法实际先遇到的该条件。后续“每条保护正常边必须留区内或去16”在此次旧proof中未被执行，不能把它记成已触达的新失败；若仅选择正确返回副本16..24，也仍需另证明去26的正常出口，这是后续设计义务。

## 实际 canonical 边界

两腿相同，normal successors顺序保持实际打印值：

| block BCI | end | 实际指令BCI | normal successors |
| --- | --- | --- | --- |
| 0 | 5 | 0,1,2 | 26,5 |
| 5 | 12 | 5,6,7,8,9 | 26,12 |
| 12 | 26 | 12,15,16,19,21,24,25 | 无 |
| 26 | 50 | 26,29,31,34,48,49 | 无 |
| 37 | 48 | 37,38,41,43,46,47 | 无 |

BCI48/49已融合进block26，真实图中没有独立block48。受限protected body的正常落穿出口是26，而外层finally的continuation是48..50尾部；Plan.join=独立48是不成立的。下一步须复用既有Continuation::Tail/受限trail接缝，折叠清理后只发射48/49一次，不能认领整块后吞掉或重走其cleanup。

## 实际值身份

- 返回链：getstatic12产生Stack0 Value20；store15读取20写Local1 Value21；load24读21产生Stack0 Value24；return25读24。
- 异常链：store37读handler Stack0 Value3并写Local2 Value12；load46读12写Stack0 Value15；throw47读15。

ValueId是本次真实SSA内的身份，不能跨方法或两次run据同号猜等价。将来证明还必须核唯一消费者、所有normal/exception边、三份cleanup及自身保护范围；上述观测不替代完整证书、负控制或停止验证。

## 行为红与二进制边界

先前旧生产native共6命令：regenerator1/0/0、正常FP5/0/1，public0/2/0与CLI0/1/0均实际行为断言红101，build与clean0。失败首先落在尚未呈现的finally/字段读，不是工具失败；后续来源断言未到，不冒称已验证全部来源。

private probe本次1通过/0失败，证明的是原拒绝和块边界；不能叫新能力测试已绿。其cfg(test)源改动后，构建CLI v2实际59,699,632B/0555，SHA256 abab495171f4f1a56ca9d4ccaddf77678a15349dfd52416e6485a037651816e4，与phase1的6e075不同，不能宣称字节一致或直接重标旧CLI收据。另以此新abab CLI实际做4条红重放：双腿完整JSON exit0，双真实javac对完整原样report.text均exit1/missing return；两份1560B文本仍SHA57926d1ddc3ad60342c853a185ff5fa3d8e8ae5c79288da0cffe65f5ad76a8c1，没有rebuilt运行。原source/driver/oracle与两class未改。
