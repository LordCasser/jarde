# 新 CLI v3 整类矩阵：末组磁盘 preflight 失败

Root 实际执行 v11-prep-v3 的12组矩阵，session26674终结1。前11组 collector 退出0、尚未做最终全矩阵独立接受；最后controls-none在第15条命令启动之前触发5GiB机器空闲空间守卫：实际 `5359357952` bytes。该条CLI渲染未启动，没有原始stdout/stderr，collector明确记录collector-error。不能把12组宣称完成、不能把部分矩阵当全量通过，也不能放宽资源门槛。

捕获归档包含私有collector/checker/runner/raw清单/所有源码diff与pins，11组完整观察和末组部分观察；原batch status failed与末条preflight异常保持。CLI SHA `45b5fd1ef637d8879254743d51ea5ce66ec5400558b023e65a7448a9d596de09`，focused v11 receipt SHA `0a2acef11bc45b2614d98b282e42842859d9502c4d211643c294db429dc31e83`。本失败是可用磁盘不足，不是已观测到的反编译/重编测试失败。

两seed完整workspace同次raw事实已独立接受，位于相邻full-workspace-raw-acceptance-root-v1；finalfmt/Clippy/strict/新CLI矩阵/lateStop及产品自身CI仍未交付。恢复空间后应在新目录重跑完整矩阵，不拼接该失败记录来冒充新执行成功。
