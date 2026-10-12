# 下一 MVP 的源码与测试方案准备

此处保存root已静态审读的Luna v2候选源及计划。Java尚未编译或运行；未创建新OpenSpec变更，也不证明当前Region/泛型恢复已覆盖这些新形态。当前两项须先完成产品CI和干净主线交付。

完整旧控制类与Runner保留全部17条原oracle调用，新增String/Integer两分支、empty/null、iterator/hasNext/next/add分别RuntimeException/Error、remove(Object)前缀和未知用途控制。v1构造器歧义风险及output.add误注入input由root发现后在v2修正。嵌套循环仍待同次实际Region/整类证据确定，不能将其拒绝作为完成捷径。

archive-pins.json核当前保存字节，原pins.json记录准备时的31项真实来源及候选源路径；迁移保存不改其历史路径。源码不是反编译输出，没有所谓“手修生成Java”成功。后续仍须双JDK/debug-none/default-all/完整未手修JADX与Jarde编译运行、实际异常与调用轨迹、源映射、预算和取消门。
