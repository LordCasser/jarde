# CF11 固定 TestNestedLoops 实际三方基线

2026-10-12，root 实际运行并独立接受 29 命令/58 raw。冻结 CLI 对应产品 `02ecd63ff33270cea4413793db2f99d6927dc65c`；最终 CLI SHA256 `51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4`，0555/61213840 bytes。完整 4718 项源码闭包同 final build receipt、运行前后/live，target 不存在。本观察不修改生产代码，不代表产品 CI 或整个 CF11/71 单元完成。

上游 TestNestedLoops.java SHA256 `4897fd18a15fce6e0abb40dc8afc6c7a3fa9f16e617f3c7e5e88072f6a37309e`。准确方法字节 `[279,573)` 保持为294 bytes/SHA256 `a0b9999d8687d5ec6ac4676d68abf84d4f1de14ebc6146fb2048d3e1735d23ce`；移除JUnit/外部owner并换为 `cf11.UpstreamNestedLoops`，不是整个上游class字节等同。Runner记录八case的异常类型和两个最终列表，不预言fail-fast行为；v2的cleanup输入顺序会触发异常，未运行原稿保留，v3使用`[four,keep]`真实验证移除后正常进入clear。

原class双JDK八行oracle一致；JADX default/rename-none完整源码四编译运行腿和Jarde default/all完整源码四腿全部同原，全部使用`-Xverify:all`，源码未做编辑。两个修改集合case真实抛ConcurrentModificationException并保留精确修改结果；三个null case为NullPointerException；正常退出后cleanup清空l1。全部原始结果见 `capture/raw` 和 `root-acceptance.json`。

Jarde仍输出两个structured while、raw List形参及`ordinary_generic_source_unproved` marker；JADX输出两个String增强for和List<String>。Jarde四profile的完整类均可编译，无行为差异或physical来源缺口。ctor3/test45物理BCI完整，方法flags/index/Code span/UTF8来源及default/all全文/map一致；BLAKE3仅核格式/location/owner/default-all一致，physical class独立SHA256，未独立重算BLAKE3。

已确认两项语法呈现差距：增强for识别和泛型Signature源码投影。`source-review-luna-v1`只是静态审阅，真实内部首拒尚未诊断，不能把raw泛型当已证循环拒绝原因。优先先处理已实测的TestSequentialLoops2尾部构造器条件值，再以同IR核foreach候选；不混入当前for-condition变更，不新建IR/pass或预先决定新机制。JADX本地源码checkout与运行版1.5.6分开记录。

`capture/execution.json` SHA256 `5635e60b5565a19ba4fab116d0b6ae73f72e62568e0761edeff2365291c95435`。Generic checker只校验观测，root额外核八个完整生成运行腿、完整源码形态、八method maps和准确闭包；原始source准备/静态review/真实运行分开保留。
