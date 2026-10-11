# Root三方对照工具审查v3（尚未执行）

保留私有v1/v2不变；root v3修正v2遗漏的1-based Jarde render起始index：前17命令均为original/JADX，实际render为18/21/24/27。v2报告声称jarde_start=17选到18不成立，不能作为正确索引证据。v2其余两个实际错误已修正：JDK tools Path误下标；physical owner缺class simple部分。

root已读原collector/checker完整关键流程：29命令真实capture，JDK8/23原完整类/JADX default与none/Jarde default与all，Xverify运行同JDK原真值；固定仅return12，受控10cases含早return/count和零迭代；BCI独立parser+javap、Code envelope/flags/order/准确identity/UTF8 map/fullclass源码重编原样与default-all相等。Python ast.parse通过只是语法准备，未运行collector/checker，freshCLI实际receipt尚未存在。

Root v4另外去掉复用Rust guard的历史EXPECTED_SUMMARIES判断及未实际应用的Rust ENV_VALUES报告：Java/CLI命令不用旧Rust测试summary，actual JAVA_HOME保留在每命令row，argv由独立checker重建。守卫5GiB/1GiB与1秒轮询保持；未执行对照。
