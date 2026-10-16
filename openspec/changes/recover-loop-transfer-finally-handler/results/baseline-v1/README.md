# 原始控制与实际IR基线

最终基线capture.tar.gz为3,544,794B，SHA256 d58609954cbda5a440a1f72873e96dd9f5cdd67647e0e654dc879d7654a4daa5；90payload加嵌入MANIFEST.json共91安全唯一普通成员，外manifest与嵌入逐字相同，逐项重开校验。CLI/JDK/Rust二进制不入包，历史绝对路径按members.source映射，完整4942文件checkout不重复打包。

独立控制12命令/24raw全0：两编译器原源/driver各编译，四次原类运行总计4×168B oracle逐字等。两份完整生成Java各2412B原样javac exit0，但两方法仍有handler45 fallback；没有生成类rebuilt runtime，编译成功不等于恢复。原两方法Code各61B与FX切片逐字等，独审accepted。

fixture native5命令全0，reader1/privateprobe1/指纹5共7 passed/0 failed/1 ignored，4942源前后一致、真实exact clean。指纹2241旧项原样+5=2246；reader实际1183/5279/545/3487/10。补充throw-sites native另2命令全0，只加一行test println，probe与此前同名不增加unique测试；两份compiled-guard快照各自与其源码清单绑定。

真实四上下文（双编译器×continue/break）各35指令/effects、三rows、8块/11边；canonical异常边16→45/row0、34→45/row1、45→45/row2保留。直接throw_sites只有5/19/52且handlers=[]；三保护范围[22,28)、[34,39)、[45,47)内实际操作均nonthrowing。handler无normal前驱，caught phi的三个来源是canonical保守Caught(16,16)/(34,34)/(45,45)，不是数组load19的保护边。保存45→局部更新47→重载50→throw52及完整uses/frames均保留；31目的地continue53/break59分别核对。正常IR/源码不修改这些原事实，后续产品证明和source-map/执行仍待实现。

包v1曾把exit0误写为0次编译，并把每腿2原运行误写4；v2数字修正后遗留一条旧说明。两历史包cache只作失败元数据审查留存；v3更正元数据并加入实际matrix独审，不改原收据/raw/源码/oracle。原prepared-only runner v1未执行，实际runner v2在执行前修正了head/JSON字段/timeout argv，不能混同产品失败。
