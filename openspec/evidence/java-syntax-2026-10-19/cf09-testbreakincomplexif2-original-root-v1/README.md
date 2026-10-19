## Accepted original baseline

完整1125B上游TestBreakInComplexIf2.java（SHAc51ba62db7e9d9d7df86bbe04fe38f37357fe62e1dc7514edc54701af94ecc92）含outer/TestCls/check，未裁剪。11有限case保留原三组check列表，并覆盖空/null列表、可达null元素、skip后null、长串/空串/a提前break；静态调用计数没有冒充实际计数。

冻结主线CLI 8c91115a…实际46命令/92原始流、八API、20完整源码重编尝试。原件四隔离VM共44观察，JADX两次JDK23 --release8完整树重编/四VM44观察均同原件。Jarde八目标均fallback、ctor/check均structured；16完整源码重编失败，无候选VM。八JDK23缺return，八JDK8保留原始框架55/52边界；JADX两个JDK8失败亦保留。

公开首gap为loop候选18的StatementFree/24，未覆盖81/46/72；来源11/34，四组default/all全文/身份/质量/maps相同。此为真实恢复缺口，还没有本用例的同轮内部IR或修复验收，不据相似消息认定已修好。

独立审计实际复开13057个pin，root另核八VM的driver-only classpath、parentless同根CodeSource、空cwd和三个错根在CASE前拒绝；source/工具输入不变且target absent。异常比较限类别，原始观察保留。审计工具的`--out`实际写入单个JSON文件，复制为归档内`actual-baseline/independent-review.json`，原实际路径与SHA由root-review绑定。

`capture.tar.gz`逐成员bytes/mode已root复开；`root-review.json`给归档身份，内含全部实际收据和v1未运行准备/v2修正。CLI/JDK/JADX及依赖为外置pinned输入，不打包工具二进制。CF09/71单元/612文件仍partial。
