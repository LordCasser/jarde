# 物理字节码与接线调查

本目录仅为read-only静态调查，actual class字节解码与CLI已有报告分别标记，不构造或推测尚未观察的SSA。两方法各61B Code/35个BCI、异常表三rows、31目标53/59及handler45来源可核；本片task1.2仍须直接观察真实P2表。

规划独审无阻断的两处澄清已纳入：JAR/WAR完整加载验证均列task3.1；原始canonical Exception边与throw_sites可触发边分开，现exception_edge_takeable不能替代完整证书，不能机械要求三份清理同一SSA ID。此处不代表fixture/实现/CI或任何task已通过。
