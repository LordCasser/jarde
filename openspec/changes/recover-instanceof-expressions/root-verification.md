# Root夹具验收起点

2026-09-23，root独立重编译源文件并与冻结InstanceOfProbe.class逐字比较，相等；1548 bytes、15 Code，SHA-256 `d8f4a437d0cd0f3ec12672791f27b1c11cfbcdc6885d10eddcd3114aa0838e56`。原始冻结class通过-Xverify:all，输出22行。helper与runner不作为永久class提交。

完整JADX fixture输出有三处javac错误：两处String静态类型与Integer目标不兼容，以及没有目标类型的method reference参与instanceof。没有执行JADX编译失败的正文，也没有改写正文后宣称通过。另一个type-boundaries独立类的四处错误属于不同输入，不能相加后写成同一类。

throw验收后的debug CLI仍有26处引用，完整fixture javac失败。root运行`cargo test --test p3_instanceof --locked`确认测试能编译，三个常规正面测试均在objectString的Mixed/Java差异处预期失败，JDK对照一项ignored。这是实施前的红测试，不是当前恢复成功；日志在`../../evidence/java-syntax-2026-09-22/instanceof/final-fixture-before/root-red-tests.log`。

该class已纳入91 class / 509 Code / 75 handler / 231 target / 8 subroutine的统一reader重扫，以及219文件fingerprint。统计验收与语法恢复状态独立；1.1已满足，1.2负面边界仍由fixture代理准备。
