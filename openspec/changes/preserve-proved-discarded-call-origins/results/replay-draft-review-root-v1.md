# Root replay draft review

root全文读取v1与v2完整diff（未变部分已读）。v1将旧基线也要求全physical BCI、report.method错误当physical identity、旧javap索引猜错、CLI路径和新测试数不符；退回改v2并保留v1。root最初指出identity应该在entry直接字段是自己的误读，已按实际JSON确认methods[].item.identity正确，entry.text和report.fallbacks确实存在；不按摘要改真实字段。

v2真实纯JSON/schema检查已通过，只声称读数据而非工具链。root执行版进一步纠正两处必须吻合实际build freeze的runner路径：使用仓库results/run-validation-build-root-v1.py而非私有原稿（root此前修了acceptance schema，实际已成功构建）；verifier SHA函数接bytes却在6处传Path，改为显式read_bytes，不增加多态hash机制。完整Stmt pop来源仍仅允许三准确delta，旧缺口显式核且candidate全physical零缺口，不放松任何来源断言。

collector/exclusive OUT使用candidate-replay-root-v2，actual26cmd成功不等于独立验收，须等verifier所有pins/raw/classes/body/maps/runtime核过。
