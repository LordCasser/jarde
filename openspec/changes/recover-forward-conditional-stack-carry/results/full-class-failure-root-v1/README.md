# 整类失败与 SSA 诊断的 root 实测记录

2026-10-14，冻结 CLI v6 实测双 JDK8/23、debug/none、default/all，两族4份完整捕获共108命令/216 raw。baseline完整17方法类的8个Jarde cells均编译运行成功且与原45行逐字一致；其8个JADX cells真实编译失败，no-runtime。admission完整13方法类的8个Jarde cells均因consume(Object)中唯一jarde_refused_body而编译失败，未运行；8个JADX cells编译运行成功且33行逐字同原。Jarde失败不可接受，任务2.1仍开放。

root实际执行独立auditor主入口：root-audit第一次21条拒绝，发现审计失败后提前读取缺失class及文件名排序误报；修正cache auditor后root-audit-v2完整4capture，40条拒绝均属Jarde编译失败/未恢复/compiled证明缺失，JADX成功与失败均完整记录。审计器修正没有放宽必需Jarde成功门。两次结果、stdout/report、所有实际编译stderr均保留。

实际consume诊断：producer getstatic0 Value4，join12两条None的Phi-input use加真实iadd12读；Stack0 Phi Value2输入Value4/Value4、replaced_by=Value4、旧值无uses。root先执行忽略诊断测试并dumpSSA/CFG，测试成功且finally cargo clean实际执行；旧guard对缓存绝对路径封装失败，因此第一诊断无有效execution receipt，失败launcher与所有raw保留。第二轮修正封装，新私有证明正例及错误值/live混合phi/无Folded/handler/topology控制测试1/0/0实际通过，守卫receipt和exact cargo clean成功、target absent，生产接入尚未包含在这轮证明验证中。

归档包括输入原Java/class/Runner/manifest、四捕获、两次root审计、诊断原raw、证明raw与receipt、实际使用审计工具/JDK manifest及CLI v6构建receipt；排除安装JDK/JADX/CLI二进制。archive-review逐一复开全部成员核长度/SHA256。源输入和Runner未改。本记录只固定失败，不宣布新能力完成。
