# 第一轮候选 v2 真实对照

Focused v1 实际 Rust 编译失败（旧 unit 19 次 refusals 调用未适配）；root 修正后 v2 实际9/0/0、CLI build成功、clean，peak462011625B，4719 source pin恒同。新 CLI 的 frozen hash/build binding 在 receipt 中。

固定 SequentialLoops2 29 命令，双JDK原类/JADX四组/Jarde default-all四组共十 runtime 腿全同10行，完整 class原样重编执行；尾部new条件表达式正确，所有物理方法BCI/UTF8 map完整。v6 checker实际因clinit定位不支持而失败；v7只补精确 static {}; 定位后实际exit0，失败保留。

控制43命令，Positive四组 same-source isolated 完整 class成功重编执行，连同original Refusals/原Runner共19行同oracle；combined和Refusals八编译失败保留，无该生成类runtime。physical gaps限于sharedFieldAndReturn/nestedConditionals/handlerAroundConditional，不能说两个完整生成类均通过。localBoundary及独立branch effect文本有恢复，但此捕获未单独重编执行生成Refusals。

Root仅接受该候选观察：v2缺Direct provenance门，尚待新增否例/公开API/Stop/回归/最终source closure新CLI与自身CI。当前源已补gate并安装canonical fixtures，不能把v2的结果作为当前源的最终验收。
