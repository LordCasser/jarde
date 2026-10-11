# 确切产品自身 CI 验收

产品993e9f8c0f413bb7c31538e6f7c2c92a742e7af4，run38176302604，[自身CI](https://github.com/LordCasser/jarde/actions/runs/38176302604)实际四jobs/52steps全部success。capture-root实际三GH命令exit0，root独立verify-v3实际21:59:46–21:59:51 UTC exit0；acceptance SHA25600062be2293871c88f9bb6ccd6b2e7128143cca4ff99bf494d4954f8877c4b68。

双seed各362target/3480-0-97、各3577逐named状态同最终本地fresh workspace，19required全通过。固定seed只是运行输入，不把两条CI合并成一次本地回执。产品Git/live/final CLI8bdb868…与4732历史输入闭包、regressions-v6/workspace-v4/final-build-v5绑定完整，未借前片或文档CI。

v2验收实际失败因为七个Cargo examples源不在历史4732闭包。v3只对metadata精确七个example补充规范路径/类型/无symlink/product Git=live SHA核验；输出明确historical_local_closure=false，不回写或编造历史pin。全部原本pin、双seed全目标/named比较保持严格。这七个源由产品自身CI真正编译；补充核验不证明它们曾被本地历史closure记录。原v2失败raw和v3静态审阅保留。

manifest记录copy与original字节/哈希，原回执绝对路径不改。这里只关闭自身CI任务3.3；3.4必须关闭文档实际推送后运行主线clean auditor。
