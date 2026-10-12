# 夹具登记与 census 永久回归的刷新验收

focused v11 实际八条命令全部 exit 0。Root 另重读每个原始 stdout/stderr 的 SHA/长度，精确 argv 顺序和具名结果；API 7通过/1 ignored、jarde-java library 375通过（四个新增 census 各一遍）、reader census 1通过、fingerprint 5通过/1 ignored；最后 CLI build 成功并冻结 root-v3，finally clean 0，target 不存在，4822来源在 fmt前后与执行结束完整一致。默认 Rust 栈、5GiB/1GiB/一秒进程组守卫保持。

新CLI 61,924,480 bytes、0555、SHA `45b5fd1ef637d8879254743d51ea5ce66ec5400558b023e65a7448a9d596de09`，与旧 root-v2 的 `bf9757…` 不同。新增永久测试和reader/manifest元数据没有改变已审生产逻辑，但不能用旧 CLI 的整类对照替代新字节身份；任务3.1需刷新实际全矩阵再关闭。此次证明2.1永久测试与组件完成，不能替代fresh全workspace、自身CI或主线交付。

原 focused v10 的产品目标实际全部成功，但 P5 `--nocapture` 输出在具名行和 `ok` 之间插入多行清点，记录器只解析到五条名字。验证器保守拒绝，CLI build 尚未执行；finally clean和来源稳定。失败原件保留。私有v6只将P5改为标准捕获，仍要求六个具名结果，不降低门禁；v11从新目录完整重跑，不拼接旧结果。

captures.tar.gz 含两个完整真实执行目录；tooling.tar.gz 含v5/v6全部源码和pins。原receipt绝对路径不改写。summary.json是root另查后的简要汇总；不是完整工作区验收。CLI外置原件保持在 `/private/tmp/jarde-cf11-structured-list-candidate-cli-root-v3`，历史 root-v1/v2 均保留。
