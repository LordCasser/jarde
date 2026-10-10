# EM23 嵌套字段更新：实际完整类族基线

root 已阅读原 JADX `arith/TestFieldIncrement2.java`、准备的完整 504 行 collector v1、v1→v2 全部改动及原始 fixture。v1 静态审查发现将 command metadata 纳入运行三元组比较的错误，未执行；v2 限定 exit/stdout/stderr 比较后，于 **2026-10-10 05:28:01 UTC** 实际执行，退出 1。脚本 SHA256 `ea08111c1b58ca44477749a616884dc3b012cd075614f834a1f49f21dba1e1c6`，完整执行记录见 [execution.json](baseline-execution-root-v1/execution.json)。

`baseline-root-v2` 实际有 33 条命令、126 个闭合文件；两条 original 与四条 JADX 完整源码重编及 `-Xverify:all` 运行成功，stdout 均为 `add=8\nmultiply=20\n`，stderr 为空。四条 Jarde 完整源码均 javac 退出 1，没有执行运行命令；失败原样保留，不能计作运行成功。独立全文件/成员/来源验收尚待执行。

Jarde 四次 outer 请求均 `member_family=prepared_static`、`projection=projected`，私有静态 A 已拼入完整 root 源码。`test1(I)V` structured 且保留 `this.a.f = this.a.f + arg1`；`test2(I)V` explanation_only/fallback，在 `dup@4` 留下 `jarde_refused_body()`。四份 compile stderr 均拒绝该未定义方法。缺口集中于准确 `imul` 的同一次 receiver 字段更新，不是内部类未折叠。

原 javap 中 test1 在 BCI1/5 各读取 a；test2 在 BCI1 读取 a 后由 dup@4 给 getfield f@5 和 putfield f@10 共享 receiver，imul 在 BCI9。JADX default/none 都输出 `+=` 与 `*=`；现有 Jarde 同 dup 证明可扩乘法，不需按文本结构等价合并 test1 的两次读取。精确算法审计见 [audit v2](architecture-audit-luna-v2.md)；v1 对 JADX 主路径的错误归因保留供对照，不作为最终算法结论。

实现范围与验收见 [recover-int-field-multiply-updates](../../../changes/recover-int-field-multiply-updates/design.md)。当前只完成规划，产品未应用；71/612 分母及 EM23 整单元状态不变。

05:51 UTC root实际独立verifier v3退出0，接受126闭合文件/33命令，原2/JADX4完整源码成功，Jarde四编译失败/零运行；全部physical成员、OriginSet primary+derived BCI并集、outer/A来源和默认/all正文核对通过。接受文件results/baseline-independent-acceptance-luna-v3.json、执行raw在results/independent-execution-root-v2.json。v2首次实跑在constant-pool误识别为member处失败，版本/raw保留；v3只限制javap class-body解析边界，无产品修改。本片tasks2/7，实施仍未开始。
