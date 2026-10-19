# 当前产品 native 与自身 CI 完整验收

产品 `79385b75a3fb0287f9cc545eb756ee8923e46583`，冻结CLI SHA `8c91115a91a31b041ecaf53a2b964bb57606342d04b524035293ae0b7accd285`。CLI构建3282来源前后零变化，native5235来源与构建并集5242逐Git blob/现场bytes/mode绑定。CLI仍为外置0555执行件，未入归档。

一次fresh native实际391 targets/102 batches/3805 passed/0 failed/98既有ignored；60精确门与九相邻门均各出现一次并通过，含finThrow/nestedFin与循环相邻回归。默认Rust栈，5GiB余量/target1GiB每秒guard，峰值850873047B；所有guard未触发、源码不变，finally exactclean后target absent。相邻reviewer v1身份格式错误及v2修正/实际只读续审全部保留，没有重跑测试。

本产品[自身CI39200895508](https://github.com/LordCasser/jarde/actions/runs/39200895508)全部四jobs/52独立官方ZIP步骤接受。两个固定seed各391 targets/3805-0-98、105精确门，全部3903具名结果逐目标一致。119条采集命令绑定26源码/87fixture，独立官方ZIP原件、每步日志、产品Git/CLI pin与所有继承门均核验；本次无transport失败，不以聚合日志补充步骤证明。

主spec追加1requirement/4scenario，原有bytes保持前缀，delta非空行逐字相同，仅补分隔空行。实际18主规范与全部384项strict通过。当前归档包含552regular payload+inventory，root逐成员独立复开bytes/SHA/mode；大小/SHA及接受标记见root-review.json。共享文档后继提交不冒称另跑产品CI；最终clean-main在外置独立收据复核。此片交付不关闭CF09/71/612，也不表示下一ComplexIf已修复。
