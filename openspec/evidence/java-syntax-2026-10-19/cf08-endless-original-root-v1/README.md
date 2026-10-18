# CF08：TestEndlessLoop 完整原始类验收

Root已接受未裁剪的上游原始外层/目标类、8API32structured方法、全部完整生成源码及独立对照。既有机制足够，无生产修改或新增OpenSpec；CF08整项与71/612计数不增。

原始test1/test2/test3都只比较this的自反身份，没有可达退出或副作用，确实不终止。驱动从未调用这三个方法；它只在parentless loader下构造目标并核确切方法/字段库存。所有成功产物由JDK8/23以-Xverify:all运行这个有限驱动。原始4VM4构造观察另计，不能解释成循环运行等价证明。

| 范围 | 实际结果 |
| --- | --- |
| 完整生成源码 | 10源格×两编译器=20尝试；18成功、2JADX/JDK8框架major55/52失败 |
| Jarde | 16整类成功编译；32VM32构造/库存观察，同原始 |
| JADX | 2整类成功编译；4VM4构造/库存观察，同原始 |
| 错根控制 | 18生成目标均拒绝；原始错根控制另计 |
| Jarde静态循环比较 | 16目标×3方法的完整Code字节、BCI、max_stack/max_locals、异常表、访问标志同原始 |
| JADX静态差异 | test2/3从6B do/while变9B while；纯this恒等条件/空体静态等价，明确不是字节相同 |

95条实际采集命令来自独立的20命令baseline、24命令初始生成、51命令resume，190份raw。初始collector的physical模块名被局部dict覆盖，后17成功编译树的物理解析报AttributeError；独立重读已有18树后，只补17目标的34VM和17错根。没有重编已完成源码或重跑首格，未冒称fresh连续矩阵。最初auditor的require初始化顺序失败、resume对扩展facts pin误拒绝的零VM失败及审计schema更正均保留。

两份独立parser不保留Code内StackMapTable，因此不声称frames字节相同。静态循环Code证明与有限构造观察分开；上游JUnit方法没有执行。Root另核18个生成构造器访问标志确为public ()V。

[capture.tar.gz](capture.tar.gz)、[库存](capture.tar.gz.inventory.json)、[root逐字复开](capture-review.json)保存333payload加1内嵌库存，包括全部实际raw/source/class、collector/auditor历史及许可证。canonical root-review及root generated-audit-v1是正式权威；旧稿不替代新收据。baseline SHA02557fcf…，初始generated SHA27be145c…，resume SHAa1a69edf…完整值见包内执行收据/库存。CLI仍为已验收96e534b0产品，不因证据文档提交声称重跑产品CI。
