# 本片确切产品 CI 验收

产品 `5f5fb2c605d61e18f392bb3abe6bdae6e10bf41e` 的 [CI 38197592542](https://github.com/LordCasser/jarde/actions/runs/38197592542) 实际全部成功：4 jobs/52 steps。root 使用完整原始 capture、最终 CLI、本地363 targets具名结果和两个CI seed逐目标比较，独立 verifier v5 实际 exit0；[acceptance.json](acceptance.json) SHA256 `c1306764d338ef6e12f6a3d4d08606616c281f1ab123f0898108f0b24b49bf04`，status `accepted_local_own_ci_observation`，4784来源输入绑定实际产品Git与CLI。

原 v4 验收器因 verify_final_build 的三处未定义 recorded_runtime_cli 名称实际失败，原脚本与实际 traceback 全部保存。v5仅将三处引用改成前面已完整核准的 recorded_candidate_cli，没有删除检查、修改产品或重跑 CI。accepted-verifier-invocation-v2 保留真实 argv/exit/stdout/stderr，capture 与 capture-invocation 保留所有网络采集原始结果。当前只接受自身 CI；干净main交付还需实际 post-docs HEAD 外置 audit，准备脚本不是验收结果。

固定 CF11/71单元/612文件完成数不增；泛型调用双JDK手工诊断属于下个任务的缺口证据，不算产品修复。
