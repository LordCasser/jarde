# 实例数组基线 root 复审

实际采集 v3 为31命令、8完整类集合腿；原/Jarde各2/2，JADX0/4语义成功但4/4编译成功。root独立执行 verifier v7接受闭合 inventory、固定CLI/metadata/JDK/JADX版本、javac23 jar成员字节、完整未修改生成源码与Runner、全物理4方法/2字段/flags、原构造器数组/调用/field BCI及source-map完整method/owner、两次实例身份/顺序和不同RHS反例。

prepared v1未执行。root执行v2失败于JDK8 javap无hex flags；v3失败于输入不是argv末项；v4失败于Jarde class_bytes为BLAKE3而archive为SHA-256；v5失败于method identity直接name/descriptor、field identity才有member；v6失败于JADX package class目录与运行classes根不同。v7修正这些读取约束后接受。完整失败脚本保留，无修改实际采集raw或条件放宽为通过负例。

owner一致性和完整原input argv/SHA绑定已核，不声称本verifier自行计算BLAKE3。所有method来源逐项等于JSON对应完整physical identity，不能只核name/BCI。
