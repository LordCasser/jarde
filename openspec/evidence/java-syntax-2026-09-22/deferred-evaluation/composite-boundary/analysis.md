# 复合表达式跨独立语句：首次实现验收反例

root于2026-09-23使用CLI SHA-256 `feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350`独立执行，前后hash一致。DeferredComposite patched class SHA-256 `33aa4c3a213ae1aded1b72113efa85b20bda273106e07314189fb4579b4c3774`。从普通javac的4个完整Code中，在最终ireturn前插入现有DeferredSupport.mark调用；完整attribute匹配与精确before/after Code在summary.json，原class通过`java -Xverify:all`执行15项。

四种链分别是`value()+1`、`-value()`、`field+1`、`a/b+1`。mark会记录trace、修改字段或抛出同一异常对象；同时有call失败、marker失败和零除对照。原class/JADX完整源码编译和执行一致。jarde完整源码零引用且javac成功，但15项中11项与原class不同；完整原始结果及jarde/JADX结果保存于本目录，没有修剪或手改恢复正文。

首次实现只按SSA值的直接reader判断边界。call/getstatic/idiv先被iadd或ineg读取，reader位于mark前，因此不保存；纯运算结果不在当前binding_producer候选中。最终return递归展开整个表达式时，叶子的读取、异常和调用就被推到mark后。这是已有design要求的组合式求值位置证明未闭合，不是新增语法范围。

root另在`../root-acceptance/`独立复跑构造3项、直接值18项、检查24项、数组分配12项和分支/前缀12项，共69项全部零引用、完整javac及运行一致。该进展不能抵消本目录的11项错值。task2.1与2.5已退回，整体实现尚未验收；生产/Cargo窗口交原Luna worker补齐。
