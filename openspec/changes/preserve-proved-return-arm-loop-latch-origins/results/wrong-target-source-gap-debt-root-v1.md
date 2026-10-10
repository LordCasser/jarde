# 无循环If尾transfer来源独立债务

CF07 analysis-only把唯一goto25→5改为25→28，消除自然循环后呈现普通If。冻结If CLI实际class-source/all显示body无while、physical25无source；永久新来源测试第一轮误要求已有source而14/1失败，root复核旧CLI后修正为拒绝新增虚构while来源并保持该既存缺口，第二轮15/0。该反例验证候选前置消除，不虚称进入loop helper。旧CLI执行/class/准确唯一opcode偏移/全map保存在wrong-target-old-cli-root-v1；对mutated class只分析、不运行。来源覆盖缺口另修，不扩大当前return/latch loop片。
