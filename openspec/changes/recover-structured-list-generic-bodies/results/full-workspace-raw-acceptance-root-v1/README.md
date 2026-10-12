# 双 seed 工作区：完整同次 raw 事实接受

Root 审阅原 v3 与只读接收器的全部差异，实际执行独立接收器退出0；另一次 root raw pairing 先验得到相同逐 seed totals。原收据 SHA `a230c1a4de86c3d92cc55799bd36be586eacbcffae81f1b5ee8b3df6e01d52c8` **仍是failed**，唯一错误是把刻意ignored的诊断当作required ok。接收器只接纳此确切路径/SHA/error及原来未写入的汇总字段，不能泛化接受其它失败。

从本次195条命令的390份原始流，重新配对 Cargo metadata、compiler-artifact executable、stderr Running 和具名 test/summary；每seed全部364个目标各一次、3502/0/98，合计7004通过、0失败、196忽略，728目标结果/7200具名结果。新API7/1、Java lib375及四个census、reader和P5两seed准确匹配，5GiB/target1GiB守卫、默认stack、4822source同字节、finally clean均核准。原始捕获位于相邻[汇总失败记录](../full-workspace-bookkeeping-failure-root-v2/README.md)，未重写、未拼接历史运行。

独立验收 summary SHA `638d99f2f718d9bc4c71b6f547ce187b104be8dd9e0d2562f4ac169bbfe3c130`，明确 raw_execution_result=all_same_run_targets_passed、execution_runner_status=failed。本目录支持完整本地workspace测试事实；final fmt/Clippy/globalstrict/CLI字节、整类矩阵及产品自身CI另行验收，不能由此宣布全部交付完成。
