# 真实 SSA/CFG 子诊断

冻结 lastIndexOf 的测试打印 raw 实际在root-v3通过：1 passed/0 failed，132 source pins仅临时region测试有差异，随后原字节恢复且cargo clean，target不存在。Value9由load@0定义；Value3才是Entry(Local3)。计算栈值11仅被Store@3消费，Store局部值12仅进header phi；phi8的inputs为12/21，slot4读取包括return leaf@19，update@22经goto@25回header@5，leaf和natural loop范围不同。

root-v1 scanner API、root-v2流路径记录失败均保留并有failure reconciliation；v1 facts误混entry和load定义被root拒绝，后续typed verifier的regex/annotation失败保留。最终facts-v9完整绑定typed def/use/phi/slot/loop/leaf/frame/edges/源码恢复与命令raw，leaf与root都实际exit0。这里只接受SSA/CFG子证据，Builder实际语句仍待后续同IR合并诊断，task1.1此时不关闭。

脚本/绝对路径与所有raw按原字节保存，归档不改写原执行证据。
