# 同一真实 IR 的 Builder 初始化诊断

root-v2 exact真实reader测试1/0通过，同一次IR包含SSA/自然环/return leaf/frame/update及Builder输出。Store@3产生空Declare后准确Assign(arg3@0 - 1@1)@2，statement锚3；program7/raggedfalse，deferred/plans/bindings/rejected/refused均空，当前whole tree仍While。既有Builder rposition排除空Declare并复用准确Assign，无需新初值表达式重建。task1.1接受，2.x未实施。

root-v1两处test-only API类型编译错误raw保留。最终132输入仅两临时测试源变化，原字节恢复到bafb，cargo clean成功、target不存在。独立v1曾通过；root审查发现runpy返回namespace副本导致解析函数仍读旧raw，v2仅修函数__globals__，实际重新通过同IR typed SSA/Builder绑定。

候选按完整patch/pins归档；重复base/candidate全文可由精确bafb Git blob+patch重建，避免重复保存数MB相同源码。私有原文件仍保留，执行raw和SHA不改写。
