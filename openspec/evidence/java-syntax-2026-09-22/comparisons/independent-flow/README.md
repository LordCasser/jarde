# 数值比较的独立完整类对照

主代理独立构造 NumericFlowAudit，5 个语法方法及构造器；所有依赖效果放在 source-only NumericEffects，避免将还未恢复的 throw helper 混进待反编译类。javac 23.0.1 `--release 8 -g:none`，每次 java 均带 `-Xverify:all`。

`before/` 是当前数值比较实施前的真实结果：242行原class输出，实际jarde完整类有10处bytecode引用、javac因5个方法缺返回拒绝；JADX完整输出可编译，11行错值全部来自 `!(a >= b)` 在任一操作数为NaN时被改写为普通 `<`。生成的package和正文未改，只给source-only helper/runner添加相同package。

242项包含36组两侧NaN/无穷/正负零/最小正值的三个方法（108行），125组long边界的嵌套分支，7个可终止double while输入，以及左右生产者分别抛错的2行。记录返回值、调用trace和异常类型/message；不从比较文本推断运行正确。

修后使用 `python3 /tmp/jarde-numeric-flow-audit.py after`（本目录保存同内容的 run_audit.py）重放。验收必须编译实际完整jarde输出并逐字比较全部242行，不删除失败方法、不拷贝原方法替换。当前尚无after执行，不能把准备好的断言当作已通过。
