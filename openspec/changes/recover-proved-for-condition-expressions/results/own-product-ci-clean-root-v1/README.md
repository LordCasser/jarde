# For-condition 产品自身 CI 与主线 checkpoint

产品 `02ecd63ff33270cea4413793db2f99d6927dc65c`，自身 [CI 38154998619](https://github.com/LordCasser/jarde/actions/runs/38154998619)。Root 实际 capture 三条 gh 命令，独立 verifier 实际 exit 0，接受四 jobs、52 steps、两个 seed 各 360 目标/3461-0-97。每轮全部 3558 named outcomes 与 fresh local workspace v4 对应；两个被平台掩码的 public labels 按唯一同 status 的本地 outcome 绑定。4718 个完整源码/fixture/include 输入与产品 Git blobs、live、finalbuild/冻结 CLI 全部绑定。

`acceptance.json` 是实际验收；`ci-capture/` 保留 summary、原始日志与 capture receipt。`product-clean-audit/` 是产品提交后的实际 clean checkpoint：仅 main，HEAD/main/origin/远端一致；15 物理 checkout 仍存在，14 辅助 detached、main 祖先、干净且无 target，不占分支。不是关闭文档提交后的最终 checkpoint。

大于 1 MiB 的 clean raw 使用无时间戳 gzip 保存；`archive-inventory.json` 记录压缩文件与完整原始字节 SHA/长度，解压后逐字节核验。原 receipt 路径保持原始外部执行位置，不伪称在归档路径重跑。关闭文档推送后还需按真实新 HEAD 在仓库外执行最终 clean audit。整个 CF11/71 单元仍在推进。
