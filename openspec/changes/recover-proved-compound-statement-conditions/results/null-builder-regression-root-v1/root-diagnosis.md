实际workspace v2前49批通过，第50批NullThenBuilder完整源码精确比较失败：旧嵌套if合并为新CompoundIf短路AND，仅该条件结构改变；所有其余正文一致。必须保留原CLI9 frozen baseline/class，不泛化忽略文本差异，最小修改永久目标预期并重编运行完整class验收；其余七unsupported source controls及物理成员/map身份检查不放宽。

已root逐字重建并应用Luna private test-only唯一变换；frozen source/class不变，其他七拒绝控制和整类/成员/maps断言全保留。最终CLI v2再执行19实际命令，双JDK原/default/all/JADX八运行腿全部true=7/false=6同原，-Xverify:all/完整3成员编译集合/唯一nested→AND文字改动经独立v2接受。v1验收器JADX package class-path错误在执行前审查发现、仅private v2精确纠正；生产未调整。永久Rust回归由fresh workspace v3验收，不借此运行代替。
