# 四工程联合交付证据

本归档以 `d3e2b2457338b61660b842f5705b225bba08a73a` 为产品；最终文档/main HEAD 单独审计。`manifest.json` 逐项记录唯一相对路径、字节数与 SHA-256，根已重新打开全部成员逐字比较。保留执行回执里的原始绝对路径，并以归档前缀映射缓存目录；不修改历史回执。

- focused-v3：10命令/20raw，462通过/0失败/2忽略，fmt、当前CI Clippy、两版本strict367/0、实际CLI build和finally exact clean成功。
- workspace-v3：195命令/390raw，默认stack，两seed各365 suites、3587/0/98，共7174/0/196；每seed77具名门一次通过，最后target absent。本地Java25/javac23组合不冒称CI Temurin。
- 产品自身CI 38673548915：4 jobs/52 steps成功，native Rust1.99/Temurin25.0.4+7/Node22；两seed各3587/0/98，strict367/0。完整API捕获与logs.zip、根654checks及独立1173checks保留。
- matrix-v2：实际运行于c814 CLI，双JDK/debug-none/default-all共16 Jarde完整类单元，baseline45/admission33行逐字同原；JADX八baseline编译失败/no-runtime、八admission成功。d3重建CLI同59602720字节及SHA cdf06694fafcb4e8cef5a4b9452676489e8cf02495c0254ce38a21e44ee4c7d5，因此按二进制恒同复用，不声称d3重跑16单元。原Java/Runner未修补。

historical compiled-source closure为4887项；额外13个fuzz目录遍历输入仅audit-time sidecar，不倒填为运行开始指纹。实际P5测试核全2219 corpus项，根也重哈希全部2219。原canonical/emitter/c814 CI、workspace-v1/v2失败及真实P2计费观察和corpus标准再生成全部按原scope保留。canonical新增计费用真实recorder更新四边界常数/三golden共74叶；corpus仅三BLAKE3语义值改变（标准生成器另排序九对象），未降低任何断言。

四项更改共16任务完成，规格同步/归档与主线整合的最终状态见上级verification-root及handoff；后三项新计划仍0/27，不扩展为CF11/CF18/71/612总体完成。

归档后的文档门：两版本strict363/0。首轮362/1因旧reuse-selected-class-read MODIFIED遗漏新三场景，失败raw保留；仅补齐该段的已生效主契约后复验通过，未改其余要求或任务。ci-extra-root.json逐项保存九具名测试、两example argv及十二closure日志行；closure-source-root.json证明本次收尾4900项当前来源相等。外层补充文件另列supplement-manifest.json，不改tar历史内容。
