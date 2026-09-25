# root 复核

在新工作目录重新javac并精确插入mark，patched SHA `cc8901164d3aac1206b65fc03c69d8798359b88c0bd300c5202ef614ccc79d0b` 与worker一致。原class与JADX全类编译/验证执行6项相同。固定ecab的jarde保留5处quote、全类缺返回而javac失败，方法可执行片段只有mark，无saved/local名字或重复take。完整JSON来源经断言覆盖shift@2、store@3、load@4、take@5、mark@8、return@11。该拒绝边界通过，不把它称作功能恢复成功。
