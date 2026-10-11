# Try-only最窄探测：未通过默认栈

fmt0、真实CF11 Reader/consumer/跨path与Stop单测1 passed，API默认线程仍stack overflow/SIGABRT (101)，clean0/target不存在。Try extraction将dispatcher从202512 B降至190912 B，helper11632 B；仅当前ARM64 debug。失败binary路径/hash绑定execution，新prologue绑定同binary，未借旧机码。

实际consumer complete354步，353 limit在BCI62停止；proof limit3在BCI23停止，usage均有真实stderr trace和原子状态断言。仍须更小化实际大dispatcher和完成fresh suite。
