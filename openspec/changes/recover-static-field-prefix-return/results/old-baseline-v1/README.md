# 本片实际旧基线与永久反例

实施基线为reference片交付main的fc195175d2a162a2ac8caa286a7cd5c69796d3e2。原FO.java、原新增driver和6B/60B oracle及两790B class逐字冻结。16命令基线：2FO编译逐字匹配、2driver编译、四main及四probe在Java8/23全部exact、两完整CLI report.text原样javac退出1，具体missing return在bump()I；evalThenFin已保持返回值后finally。全部5方法保留，rebuilt runtime为0。

显式regenerator实际1/0/0，2220→2226只增六输入，旧记录/分类不变；其打印状态夹在test name与ok之间导致采集parser失败，v1只执行regenerator和clean，未到后继。续跑v2 normal指纹5/0/1、public1通过/2行为失败、CLI0通过/1行为失败；库两红及CLI都在首个v8腿首断言退出，第二腿和后续来源断言未到，不能将所有计划断言记已运行。finally 控制实际比较两腿 Jarde 恢复文本通过，并非恢复源码运行通过。

CLI重新构建后与旧c44字节相同：59,657,968B、0555；Cargo fresh实际true，因为此前CLI测试已构建；这是旧生产CLI，不是修后CLI。exact clean成功且target缺失。基线完整4914=旧4907+7fixture；native收据只记selected4902（不含13个fuzz种子），前后相同，不能冒称4915完整before/after；normal指纹实际检查全2226，独审另核13个种子与旧基线相同，两个范围分别保存。

另两次受控javac手写owner-shadow对照：原控制成功，独立候选++PrefixOwnerShadow.i报int cannot be dereferenced。没有修改FO/driver或任何Jarde生成Java，不能把这条候选称为实际生成缺陷。owner合法拼写的最小准入建议另见owner-spelling-review。

capture.tar.gz：3,631,764B，SHA256 1325b7c3074859581b1b263bd90f797f2be7da6d2a6234c10185e22178ee5810。91 payload 加 manifest 共 92 个安全唯一普通成员；25 命令、50 条命令 raw，另有 12 条 launcher/dispatch raw 和 5 个 metadata JSON。准确执行过的 v1 harness 已纳入。原失败、未执行范围、工具/CLI 外 pin 及实际源清单均保留，root 已重开全部 payload 核 hash。基线独审 0 错误，旧 native 独审 75/0。此前未采纳的 v2 metadata 草稿被计数更正版覆盖，未丢失任何执行或 raw；v1/v3 cache 保留。

仅任务1.1完成；1.2永久行为红已观察，但真实证明期停止控制、生产恢复、修后完整源码运行及本片自身CI尚待执行。本证据不增加全局覆盖数字。
