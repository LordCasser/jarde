# 四组永久语料 root 固定

lambda/type-qualifier/primitive-conversions/narrow-integer-returns四份class均经独立源码重编译、需要时精确patch及JVM原class验证；helpers和runner只保留源码。root运行四组Rust测试：lambda为2pass/1预期动态检查失败/1ignored，type qualifier修正只含方法正文的signature断言后为2pass/1真实名称捕获失败/1ignored，转换与窄返回各2个Mixed拒绝失败/1ignored。没有声称这些功能已实现。

reader实际测量由94/579/75/236/8增为98class/639Code/81handlers/236targets/8subroutines；仅更新实际人口计数，重新执行通过。fingerprint regenerate后251文件，其中19新增、旧232内容与digest全部保持，无删除；常规fingerprint 5pass/1ignored。完整命令输出及diff在本目录。
