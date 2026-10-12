# v8 实际计数断言失败

真实 API 6 passed /1 failed /1 ignored，失败仅为 lateStop 旧用量期望25195而实际25299。内层 Declare init加入同一库存后 inner-option成功；null、negative与其他正例保持通过。增加的真实库存使证明工作计费增加，没有为维持旧数值漏算或过滤。root按这次实际结果更新永久期望25299，后续需fresh完整重跑。fmt/HEAD0、Cargo101、finallyclean0/target absent，4822完整来源前后稳定。library和CLI未执行。本目录保留原始失败，不冒充已通过。
