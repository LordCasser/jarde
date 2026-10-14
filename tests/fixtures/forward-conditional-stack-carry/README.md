# 前向条件栈携带值的时序探针

root 在2026-10-14使用钉定Corretto8.504.04.1与Oracle23.0.1，各自完整编译Java8 source/target、debug开启。原source/Runner仅打印真实观察；两个原Runner均12行且SHA见pins。四个default/all未手修完整16方法Jarde类均重编译运行逐字同原；root独立检查64方法maps、原/编译类完整方法库存及raw pins。实际记录位于持久forward-order-v1，并由OpenSpec最终归档固定。

覆盖字段初值10且选中分支把字段改100后最终必须11、producer/test/arm调用顺序GT(A/B)、producer一次、null arraylength在分支前抛出、RuntimeException/Error RHS未执行最终写。三个compound方法限定void单赋值；返回值dup/multiple-reader另属拒绝范围，不混入正例。基线admission/Runner保持原字节，不能用本探针代替45/33行完整矩阵。
