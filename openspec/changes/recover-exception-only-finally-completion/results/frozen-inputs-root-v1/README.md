# 真实双编译器输入固化

root实际执行8命令/16raw：独立完整类/driver与封存原完整类/runner各由javac8及23编译，四目标方法都为19B/11BCI/单row。独立类各682B，两个完整原类各907B且与原封存输入逐字相同。工具沿已冻结JDK manifest逐个SHA核验，全部Java源与命令raw保留。此步没有Java runtime，后续基线矩阵必须实际运行原oracle。
