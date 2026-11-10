# S0 同源变形与数值边界输入

本目录是 development 输入，未增加 holdout。8 个原 development lineage 各派生 A/B 两个变形，另有两个独立编写的数值 development lineage。canonical 清单为 `matrix.json`；source/driver SHA 以该文件及实际产物为准。

A 修改标识符并添加被使用的 long 局部。原私有生成稿误把 padding 放在类体，root 在首次编译前移入 run；136 组独立复核确认全部 32 个 A 变体的 max_locals 增加 2，Code 字节也变化。B 增加消费点、trace 前后效果及结构嵌套。它们不是新的独立语料单位。

数值输入分别检查 byte/short/char 截断、int 溢出和旧/更新值多消费，以及 NaN raw bits 和正负零。原 class 为行为参照，未按生成源码反过来改输入。18 个源码 case 各使用两个 compiler/两个 debug 配置，共 72 组，实际结果见 `../../results/s0-resumed-v2/expanded-matrix/matrix-results.json`。清单中的 review_note 是编译前的来源记录，不能替代该结果。

原 v1 的 8 个 holdout 及其 driver 与旧 manifest 的 16 个 pin 逐项相符，本轮未用它们修改生产规则。后续若用于调规则，须记录污染并补独立评估集。

目前没有独立的循环 phi 同时复制矩阵；handler 只覆盖具体 E2/R1/R2 形态；动态加载、反射和未知外部调用者闭合也未覆盖。S2–S5 不能凭本组有限样本宣布所有相关恢复合同已实现。
