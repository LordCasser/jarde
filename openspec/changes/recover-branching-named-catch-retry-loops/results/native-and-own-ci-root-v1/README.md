# Loop2 最终工作区、产品自身 CI 与来源绑定

产品 `618b5e20f27f94ba28ea2f9e4310648718475f82` 已由 root 完成架构、对抗性及完整对照验收。本目录只记录最终工作区/自身CI/来源绑定；[完整原件和可达控制矩阵](../focused-and-full-comparison-root-v1/README.md)、[同轮IR定位](../same-run-ir-root-v1/README.md)保留各自原始证据，不重复运行或覆盖旧失败。

## 实际结果

- 一次 fresh workspace，seed `5350648285461741569`：393 targets、103批、3823 passed / 0 failed / 98既有ignored，78精确门均恰好通过一次。5273源码pin在格式化/执行前后相同，默认Rust栈、5GiB余量/target1GiB/每秒进程组守卫未触发，finally target absent。独立审计复开212份原流。
- [产品自身CI39227711617](https://github.com/LordCasser/jarde/actions/runs/39227711617)：4 job、52官方独立步骤均成功，全部步骤有官方ZIP成员，未使用aggregate重建。两个固定seed `5350648285461741569` / `5350648285461741570`各393 targets、3823/0/98；123精确门均通过。独立Luna按完整target header/test name/status重建各3921项Counter，两轮完全相等、无重复身份。158 collector命令的316份stdout/stderr字节与SHA逐项闭合。
- 最终构建3320路径、native5273路径，5280路径并集均绑定产品Git blob与实际字节/模式。最终CLI为64,347,312 bytes、0555、SHA-256 `7e321a7f4fd6a029540ab169f79572c6ad83e74119900ac6d695290c9c055318`，与完整对照使用的CLI逐字节相同。Linux CI通过源码/Git绑定，不宣称跨平台二进制同字节。
- 一个主规格要求/四个场景已同步，独立复核后将拒绝场景的实现措辞改为可观察的路径/效果契约。最终 `openspec validate --all --strict` 实际386通过、0失败；18 spec单独严格验证为早期快照，后续措辞以all strict和audit-v2为准。

## 归档与定位

[完整原始归档](evidence.tar.gz)含666个payload、33,082,341未压缩bytes；[逐成员库存](inventory.json)记录路径/bytes/mode/SHA，[root重开回执](capture.json)记录归档身份。归档6,621,575 bytes，SHA-256 `49702b04be8670596720b5d34edc7b22807c70d9000ed56666762d59c76853b6`。root已将全部成员重新打开并逐字节、模式和摘要核对。

| 成员前缀 | 内容 |
| --- | --- |
| `native/`、`native-independent-review/` | fresh执行、212原流、root验证与独立审计 |
| `own-ci/`、`ci-independent-review/` |158命令、Git/API源码pin、4 job原日志、官方步骤ZIP、最终accepted收据、完整具名Counter独立审计 |
| `product-binding/` |5280路径逐Git blob/bytes/mode绑定 |
| `acceptance-tools/`、`native-tools/` |实际验收工具、原runner、guard及target扫描来源；保留准备快照 |
| `spec-independent-review/`、`closeout/`、`ci-launch/` |主规格v1/v2审计、386 strict原流、root最终验收、真实调用与查询历史 |

归档保留运行时原始绝对路径；跨机器阅读按以上成员前缀定位，不修改冻结回执。CLI二进制和target不入库，构建身份及来源pin已保存。

## 范围与保留历史

原件catch动态不可达；重试、同一异常重抛、条件反转及保护区外效果自身异常由独立控制例实际覆盖。四个物理错边邻例只做双JVM load-only及Reader/CFG/SSA拒绝证明。保留原JADX JDK8 framework55/52边界，不宣称任意循环或全部JVM属性等价。固定13原件7项已完整接受、6项待基线，CF09/71单元/612文件仍partial。

CI前的一次本地只读状态查询卡住超过三分钟，root终止该查询后用新路径和超时重试；原始失败完整保留于closeout，远端workflow未中断。正式collector与verifier均exit0。本目录未将查询故障算作产品CI失败。

准备材料中的`bind_product_one --acceptance`实际参数是native review，而非CI acceptance；root真实调用/绑定收据为准。准备材料旧说明误写新增门数量，实际为5 Region门+2公开门；准备pin属于历史快照，最终source-pins和Git/live验证绑定最终产品。历史文件保留原样，修订后主规格与delta以audit-v2及wording-v2回执为准。
