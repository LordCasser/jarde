# 条件 switch 完整边界基线

root 用冻结 typed CLI 完整执行 21 条命令，两原 JDK oracle 精确相同。JADX 两腿各 5 行语义差异，Jarde 四份完整类均因 fallback 缺 return 无法编译。独立 verifier root v2 返回 0，仅接受观察：runtime_acceptance/product_acceptance/cf12_complete 均为 false。

收集器首轮 19cmd 在 ending file_sha(str) 出错；JADX launcher JDK8 class55 加载失败另行保留。root 修正只让 launcher 用 JDK23，生成 Java 仍分别在 JDK8/23 完整编译运行，javac locale 英语。成功观察不覆盖失败 raw。首轮 verifier 对失败 javac 的通用 Cargo test_summary_check=None 口径不符，root v2 核实际 JVM exit/raw，不将该通用字段当 Cargo 测试通过。

全部拷贝与原 private 字节相同，JSON 中历史绝对路径不回写。copy-manifest 独立绑定所有归档文件；private 原路径仍保留供复验。测试草稿只有设计骨架，不声称 graph/API 编译或反例覆盖。当前条件片仍 1/7。
