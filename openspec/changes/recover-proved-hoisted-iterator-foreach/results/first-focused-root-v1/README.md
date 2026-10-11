# 首次实际编译失败

候选v2首次 guarded build 真实 cargo exit101：helper声明参数要求 &Declarations，而 caller传了拥有值self.declarations。root仅修为 &self.declarations 后以新runner/output重跑。尚未运行公开tests或后续 proof；未冻结CLI。guard未触发，finally cargo clean已完成。全部raw和invocation原字节保留。
