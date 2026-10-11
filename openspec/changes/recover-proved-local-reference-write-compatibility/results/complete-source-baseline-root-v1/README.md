# 完整旧产品基线：独立观察接受

root实际运行debug93/none97命令，共380原始流；`check-debug`/`check-none`独立checker均实际exit0，`root-assessment.json`只接受观察。冻结literal原类、六合法边界和29case Runner未手改反编译源；debug/-g:none逐javac argv绑定，真实Code属性独立解析，输入与工具/CLI/final-build receipt均固定。

两profile原JDK8/23的29行结果相同，包含seed、null、Integer末值、迭代器调用前后异常及效果顺序；JADX两配置双JDK各四条完整生成类运行腿同原。Jarde每profile四次combined compile1/no runtime，不能拿isolated成功称整类集合通过。debug参数控制四条isolated腿通过；none参数与CharSequence各四条通过。其它isolated编译失败、每profile四条physical map缺口均保留，不能统称为同一类型问题。

slot控制真实slot2及非重叠String/Object LVT range见checker observation；none所有目标Code无LVT/LVTT/LineNumberTable。未采集SSA reuse身份，不从源码变量名或物理slot相等猜identity。

旧CLI `/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1` SHA256 `526e2e5a6db8a798a92d491123264bbb4d7105c0c1c3c2c9495c1b316eb89dd5`；生产基底73adae72，4744源码前后恒同。5GiB/target1GiB一秒守卫未触发，结束target absent/free12281155584B。`launch`和两profile原始execution未回写；prep继续保留事前planned状态，实际状态由invocation与observation表示。

root首次打包已成功复制原始目录，随后assessment断言误取compile失败行不存在的runtime_exit键而KeyError；第二次仅用get确认未运行，没有改变任何原始输出或独立checker。此打包失败说明保留在root-assessment，不冒称为产品失败或重跑通过。

随后root发现checker-v8把包括requested evidence/elapsed execution在内的整个report比较成source/map差异。`checker-root-v9`恢复原v7的正文/source-map/presence比较，两条新invocation均实际exit0，每profile14对default/all正文与map恒同；原v8与raw保留不回写。准确接受见`root-assessment-v2.json`。`diagnosis-luna-v1/analysis.md`最后一句报告存在差异指整个report（请求证据与时间等），不代表正文或source-map变化；root实核其不同键包括rules/regions/evidence/execution等。它逐类引用真实中文javac诊断：Object→String、StringBuilder→String、CharSequence[]→String[]和handler缺返回分别记录。
