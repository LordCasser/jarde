# 自身产品 CI 证据

产品 aadf71f3b4f5658485196edb63fb553f8c107380 的 [CI 38703952159](https://github.com/LordCasser/jarde/actions/runs/38703952159) 实际完成：4 job、52 step 成功。两固定 seed 各 368 个 summary、3631 通过、0 失败、98 忽略；每 seed 121 个具名门通过。root 826 检查/0 错，独审 1549 检查/0 错。忽略测试不等于全部另行运行。

capture.tar.gz：2315516B，SHA256 974cfafc3cff3d6c26e341d30d8ab3aecd47795fc024d3acad252f6dfae79f06。314 payload 加内嵌 manifest，共315个安全唯一普通成员；含294份收据绑定raw、原logs.zip的60成员。manifest.json 是内嵌清单原样副本；包已回读核验及独审。

收据内的原始绝对路径与历史 pending 字段保持原样，不能单凭包内旧字段判断最终 CI 状态。后续仅 OpenSpec 文档提交保持4905个产品输入与冻结CLI一致；本run绑定产品aadf71，不声称后续文档head另有自身CI。局部原生及源码/双JVM证据见 ../local-v1，旧失败见 ../repro-v1。外置原生CLI/JDK仅pin，不收入此包。
