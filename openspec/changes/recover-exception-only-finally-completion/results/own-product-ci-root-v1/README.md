## 精确产品自身CI验收

产品3fa506df0ccd7da3d38ab769bfd65235817508fe，[CI39051492999](https://github.com/LordCasser/jarde/actions/runs/39051492999)实际4jobs/52API steps全success；全部workflow action/run步骤逐项对应，未过滤或跳过。两固定seed各378targets、3700通过/0失败/98忽略，9新关键测试每seed一次ok；root另逐target比较两seed五项计数，完全一致。

完整API及四job日志、10收集命令20raw、精确git-show的5关键来源/工作流pin、collector和独立checker逐成员复开。4955完整产品来源及3002构建来源另见product-source-binding-root-v1。JDK25额外对照、fmt/Clippy/MSRV、依赖层级、supply-chain及3个fuzz smoke和corpus replay全部通过。此CI只归属于产品SHA，不冒称后续文档head自身CI。
