# Tasks

> 纪律：门控实验先行（循环测试位准入单独翻转 readAll；copy/dup-store/guard 全负例不翻）；行号按锚点名重验；readAll 当前拒绝链先插桩（copy 族文本？crossing？顺序如何）。

- [ ] 1.1 插桩 IO.readAll 当前拒绝链（逐字诊断与先后）；定位 dup-store 双读者判据的循环测试位缺口的发出处；门控实验转录存证据。
- [ ] 1.2 冻结锚与负例双腿：IO（巡查件，readAll 为主锚）+ 独立探针（非 guard 体内的同形循环）；负例=store 后读者不止测试形、guard 体内不可观察性违反形。
- [ ] 2.1 实现循环测试位呈现（测试表达式内 store）+ guard 体局部声明位协调；既有判据逐字不动。
- [ ] 2.2 对照测试：readAll 恢复（IO 类 0 引注、双腿文件驱动含 EOF 一致）；countLines 零回退；copy/快照/dup-store/guard 五族套件全绿；负例拒绝逐字。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：插桩链、门控、行为对照、账本（io 域 readAll 残余关闭）。（留 root）
