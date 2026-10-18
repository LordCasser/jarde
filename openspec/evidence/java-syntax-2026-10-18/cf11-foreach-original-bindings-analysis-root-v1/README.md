# 下一固定项：原名 foreach 绑定（仅分析）

9成员，SHA256 `41e84a06e12189fd54503000c22d6c996c83dfe6944339d7f96320d0b0e7c96d`。含root物理/AST来源分析、raw方法map补核、Luna对抗审查、JADX三个邻居文件的源码pins和算法参考。

确认外层空声明/首赋值同BCI25但scope不同，foreach anchors为13/42而非next17/47。现有物理slot/SSA+opaque AST足以复用，下一片需闭合全部局部读取到词法范围的窄关系；不先新增全局solver/pass。需明确演进现有“保留body binding”约束，仅证书闭合者允许提升，其余保留typed行为。当前没有新Spec/实现/编译运行，未把静态JADX wrapped路径差异宣称为新bug，不计CF11或71/612完成。
