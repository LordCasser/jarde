# 精确产品 CI 验收

产品43dffc9b806dc2b69102e50f52ee78a50850c6d0，自身run38087978610。root v7实际退出0并接受4jobs/52steps全部成功；双seed各356summary、3408passed/0failed/97ignored。准确lib337、typed9、Meeting6、Required8、Ref8、P5五passed一ignored及86唯一路径Git/live pins均复核。

v6第一次verifier因Cargo header空白边界消费后续摘要而失败；原失败、raw和调用不覆盖。v7仅复用既有严格header/准确测试名/计数解析，冻结产品与CI日志不改。capture v6实际成功，v7复用原字节。接受范围不包含整个CF12或长期目标。

[实际接受](jarde-typed-ci-close-root-v7/acceptance.json)，[实际调用](jarde-typed-ci-close-root-v7/verifier-invocation/execution.json)。
