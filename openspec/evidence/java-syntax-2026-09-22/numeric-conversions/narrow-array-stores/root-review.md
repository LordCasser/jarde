# root 独立复核

root-948a从固定源码独立javac、应用8个精确descriptor/store补丁，重新验证并执行196项，输入SHA与worker相同。jarde有12处明确引用，完整源码可编译/执行但186行不同；普通B/C/S局部回读及boolean局部也被拒绝，不能称8个拒绝已穷尽问题。JADX完整javac失败，未执行。

core-bcs在输入源码阶段移除Z相关3个方法与对应runner调用，重新编译并应用剩余6个补丁；147项原JVM验证执行、完整JADX/jarde阶段均重新实测，未剪改任何恢复输出。该独立类作为recover-narrow-array-stores的必须恢复正例；原始Z与boolean局部继续保留边界。两个目录均固定源码、patcher、class、CLI hash、完整日志与summary，可重放。

进一步用all JSON检查真实来源：storeByteProduced/storeCharProduced/storeShortProduced的call@4在拒绝正文source-map中缺失，仅7/8；storeBooleanProduced保留4/7/8。证据为root-948a/full-evidence.json与refusal-origins.json，发生于array_write数字类型不相容分支只fallback(vec![at])。同一写入任务需沿用已有quoted_bcis闭合拒绝来源，无需新追溯框架。
