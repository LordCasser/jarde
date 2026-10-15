# 最终候选本地实施证据

本目录验收本地实现，不代表产品自身CI已通过。410个payload/411个regular tar成员；archive-review逐字节重开核验通过。capture.tar.gz为16210008 bytes，SHA256 `7eb9d5488ae6af89f10e79dfec1704852a3b9408a8533c5e393bd1c78452b37a`。tar内manifest逐payload列出bytes/SHA；原始历史manifests和失败raw均未改写。

## 实际结果

| 范围 | 真实执行 | 结果 |
|---|---|---|
| focused-v5 | 单一4902输入闭包、13命令/26raw、17 suites | 441 passed / 0 failed / 1 ignored；fmt、原样CI Clippy、双strict各365/0、JDK23版本、CLI build及exact clean全0 |
| 六档JSON CLI | 同一冻结CLI，ClassHeaders4096/1/2/8/14/15 | 实际退出0/4/4/4/4/4；wide费用16，其余费用各为本档上限 |
| 前置8/14 | actual main及原analysis/execution/诊断 | Stopped/not_produced、空正文/maps/无恢复declaration；保留原ClassHeaders，不改报Cancelled |
| 后置15 | 与wide逐六物理方法比较 | identity/index/text/完整outcome仅归一化两个elapsed叶后相同；main各有71段map |
| 完整H1类族 | 11份原text Java、全族javac23 --release8、16命令/32raw | 全部报告Complete、编译0；原JAR/重编译在真实Java8和Java23的四次-Xverify:all均逐字匹配原8行oracle |

冻结CLI：59674128 bytes，SHA256 `2669b2064747f16ff883788bdf31811b67c5ed3da2da3c1676eed6e99eec04ac`，0555。执行前后全部4902源/fixture恒同；构建后与所有CLI/完整源码回放结束target均不存在。H1原JAR4469 bytes / `1ee63f3836a5e514cea57b7349dd515ca3f98550154beff187fdca4d019105b0`不变。没有修改生成Java、裁剪方法、替换Runner或oracle，不扩大CF覆盖统计。

## 审计位置

- `evidence/class-source-proof-focused-root-v5`：唯一最终focused实际receipt/raw；`root/class-source-proof-final-focus-root-acceptance-v1.json`及independent-review核实441结果及原输入。
- `optional/six-cap-cli`：六次实际命令和所有raw/parsed，semantic-audit-v4严格绑定实际CLI与原停止/完整outcomes；root下两次auditor草稿失败说明保留，未改真实capture。
- `optional/h1-full-source`：全部生成Java/classes、原JAR、原oracle、16命令raw和source/input manifests；root下root-acceptance及independent-review分别核5025检查/实际全源码与四次JVM。
- `evidence/class-source-proof-final-static-review-v1`：原a5e到本片所有产品变更与边界评审。三个legacy邻居wrapper、全局Budget与merge优先级未改；无ledger本地期限分类修正仅静态覆盖，不冒称已动态到达。

## 历史保全与范围

initial-v1–v4、focused-v1–v4、旧生产six-cap及错误Greet对照、旧public草稿和所有source/raw/失败review均保留。focused-v3的14-target编译触1GiB guard，未执行该批native；最终拆为两个七目标，并在v5从头完整重跑。Greet是可按命名到达的终点，删/重复它不构成应拒绝的证据；最终只改实验输入到真实必读的Mid，保留拒绝断言。预算/取消/期限通过真实收费、checkpoint、原report两渠道交接；不伪造Error、不预取消、不sleep。

当前仅本地范围通过，workspace/MSRV/最终自身产品CI继续待验；不借历史class-view e8c或接口cc305结果代替。接口最终同CLI完整源回放另行留证，不计作本片新增源码族。
