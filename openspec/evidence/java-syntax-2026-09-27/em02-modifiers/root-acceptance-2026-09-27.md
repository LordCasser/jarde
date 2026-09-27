# EM-02 主线独立验收

root 审阅直接父方法关系与最终文本投影后，将实现合入主线并重新构建 CLI（SHA-256 `15ddf7d89aff21db8943e6cebe3451873e7c3bb66816437cad5500afecad7937`）。独立运行固定 [replay.py](replay.py)，JADX 提交及四项测试哈希均一致；原 class、JADX、Jarde 六个完整类源码以 Java 8 重编，`java -Xverify:all` 均输出 `7:5`、`1:2:1`、`2:1`。日志、源码与摘要保存在 [root-replay/](root-replay/)。相对冻结基线，Jarde 唯一源码变化是 `PackageChild.onlyHere()` 前的一次 `@Override`；私有父方法和跨包包私有同名方法均未误加。

代码仅取请求已选同 jar、同包的唯一直接普通父定义，复核父/子完整物理成员表、准确同名同描述符、实例方法及合法可见性；Signature、bridge、synthetic、歧义或缺失关系不猜。提示在其它投影之后一次性写进最终源码，并单列子/父物理方法身份，原物理正文和注解事实不变；输出预算停止不发布半份提示。主线 `cargo test -p jarde --test class_source`（85 项）、`cargo test -p jarde-cli --test class_source_cli`（17 项）、workspace check、fmt 与 OpenSpec strict 全通过。接口、多级及跨包 public/protected、泛型/协变、Smali 非法 flags 仍是独立边界。
