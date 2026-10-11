# 实际 focused 与 candidate CLI

Root v3实际15命令全部exit0/guard未触发；12个test invocation共422pass/0fail/2ignored（含一次预期manifest再生成、同corpus目标重复，不能当唯一workspace人口）。新API六tests通过，包括固定外for内while、11producer positive、local/shared-call边界、旧fallback边界及lateStop/预取消；353 Java-lib、numeric/foreach/continue/gateway旧控制、P5 manifest和Reader真实census sweep通过。Fmt通过；freshworkspace/CI等价lint/自身产品CI仍待。

Corpus manifest再生成是明示单文件变化，before-input另存pre_regeneration_source_pins；source_pins_before/after是再生成后4718项完整CLI输入闭包，完全恒同。候选CLI61213840B/0555/SHA51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4在private/tmp；不把binary入Git。5GiB free/target1GiB一秒process-group守卫，target峰710170385B，cargo clean实际成功，target不存在。

v1 prepared未运行；v2 preflight未区分两个Rust文档中的include_bytes!(../tests/fixtures/…)示例，发生assertion且没有Cargo命令。v3只排除两个明确doc示例（其他missing input仍失败）后实际通过；失败wrapper完整保留，不称v2构建失败或产品失败。
