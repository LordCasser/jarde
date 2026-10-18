# 工作区采集计数绑定债务

已确认的工具问题，独立于本片Rust语法恢复产品。

`recover-branch-throw-return-finally/tools/workspace_runner.py:310` 把compiler-artifact发出顺序与stdout harness摘要顺序zip；前者不代表Cargo执行顺序。当前第一seed首批实际先运行jarde库244项，再运行7个零测试example，原表却把244项绑定给inspect_class_header。全批次总数和383个target清单没有丢失，但94批的逐target绑定不同，不能引用原表证明逐target计数。

root-v3独立验收保留原execution/raw，先从artifact/harness逐字重建原表，确认原记录没有改写；再从stderr `Running` 的完整可执行名映射回同次compiler-artifact物理target，按真实执行顺序绑定harness的全部具名测试与摘要。资源、环境、全部metadata target、总数、来源和exact clean仍分别核验。输出单列原表与正确表差异，19项具名门必须在精确target下唯一通过。

两seed均已由该验收器重建接受：各383target、3726/0/98、5051来源pin、19门通过，逐target计数相同，target absent；旧表与实际关联分别有94/90批不同。workspace归档内的本债务快照写于第二seed审查前，其“尚待”措辞为历史状态，终态以同包root-paired-review.json为准。本片不改旧采集脚本，不把测试工具债务混入Rust产品；后续单独修正采集脚本的摘要关联方式。最终证据须携带root-v3脚本、原表和重建结果。
