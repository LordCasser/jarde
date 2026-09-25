# 既有整数条件前缀的控制样例

在数值比较新CLI重建前，用已有int比较构造 `boolean less(int a,int b){return a<b;}`。
当前jarde实际保留 `if(arg0<arg1){}` 条件前缀，并引用BCI10的0/1汇合返回；整体javac失败。
原class与实际JADX完整输出的三次 `-Xverify:all` 运行均为true/false/true。

这证明通用区域的既有协议允许已证明条件前缀与未恢复返回并存。numeric的booleanMerge负对照
应验证最终汇合仍Mixed/Fallback、有明确拒绝及来源，而不是要求整个正文为空、强制比较BCI
必须被quote，或为了使旧测试通过专门禁止该比较条件。布尔汇合的完整恢复仍是另一个问题，
不能据存在合法前缀将整个方法称为Structured。生成方法体未删改；JADX只为source-only Runner
匹配其package。
