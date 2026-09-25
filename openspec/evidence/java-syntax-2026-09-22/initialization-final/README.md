# static final 初始化的独立缺口

原样例 javac --release8 -g:none 编译；jarde debug实际整类输出直接再编译，得到三条错误：两个blank static final字段的限定赋值 `FinalInit.COMPUTED =` / `FinalInit.OBJECT =` 被javac拒绝，另有已独立规划的static块return。ConstantValue的CONSTANT=5和构造器内this.instance赋值为正常对照。原程序执行值见original.txt。

jadx1.5.6实际生成整类在其defpackage内直接编译，反射runner仅用不同类名进入，两者四行结果一致。没有去掉jarde的return或改限定符后冒充实际成功。生成源码、日志及runner均随目录保存；class在/tmp/jarde-init-final，未增加永久语料。

这项需要保留原字段初始化次序，不能因为格式错就把任意clinit效果上提字段声明。FieldAssign现有带receiver文本解释了限定名来源；未来需审查方法声明、当前类、字段身份及命名作用域事实，决定无歧义简单名赋值能否复用现有Assign，或应怎样表达字段限定方式。不得用空Path、删字符串前缀或硬编码final字段名；还要覆盖字段与恢复局部重名。尚未创建实施change，不混入recover-static-initializer-completion。
