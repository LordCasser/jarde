# 静态int前置递增直接返回

FO.java及两FO.class逐字冻结自既有return-effectful-finally实验；StaticPrefixProbe.java为规划期新增的独立driver原件。原源码、driver和oracle不在恢复实现中修改。v8为javac23.0.1 --release8 -Xlint:-options -g:none；v8-javac8为真实Corretto8u504 javac -source8 -target8 -g:none。受控工具完整版本与SHA见本change证据。

两bump()I的Code分别b20007046059b30007ac与b20002046059b30002ac，CP编号不同，均为FO.i:I；字段声明static、非final/nonvolatile。物理BCI读0、常量3、add4、dup5、写6、return9；无异常表。必须证明dup的两份新值各自的独占消费者，不能仅按上述字节猜测恢复。

main.stdout为原FO固定输出1/100；probe.stdout为原独立driver六行60B输出，覆盖-1/7/MAX初值、int溢出和return-before-finally。完整FO生成源必须原样重编，不能删bump或修改生成文本；输入可运行不等于恢复成功。本片旧CLI的完整FO当前仍因bump无return编译失败，evalThenFin本身已恢复。
