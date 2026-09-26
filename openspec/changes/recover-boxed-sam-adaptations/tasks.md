## 1. 冻结形状与三方基线

- [ ] 1.1 自写 Java 8 fixture：`Supplier<Integer>`/`Function<String,Integer>`/`Function<Integer,int[]>` 站点各一、串联两站点的方法、`int[]::new` 使用点；javac --release 8 编译冻结 SHA；记录 indy BSM 参数（instantiated/impl 类型）、原 class 执行输出、jadx 1.5.6 输出、jarde 修前整方法引用。

## 2. 配对证明与呈现

- [ ] 2.1 `lambda.rs` 适配证明加入基本/包装逐槽配对与数组构造器引用特例；arity/类型对/静态性/impl 可读性负例全部保持既有拒绝码。定向测试：正例站点呈现箭头、负例逐类拒绝。
- [ ] 2.2 使用点方法恢复为含箭头调用的语句；`int[]::new` 的合成分配方法保留并带「已在使用点呈现」标记；来源 BCI 锚点保留。

## 3. 对照与门禁

- [ ] 3.1 恢复文本 Java 8 重编译执行：边界 Integer 值、null 接收者、正常路径与原 class 一致；jadx 对照记录。
- [ ] 3.2 复跑 lambda、functional-receiver、execution_comparison 回归；`cargo fmt`、`cargo clippy -p jarde-java -p jarde --all-targets -- -D warnings`、`openspec validate recover-boxed-sam-adaptations --strict`；golden/语料计数若变重录并说明。
