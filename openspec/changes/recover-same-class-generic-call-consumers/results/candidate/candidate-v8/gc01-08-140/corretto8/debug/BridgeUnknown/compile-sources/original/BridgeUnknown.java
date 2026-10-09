class BridgeBase<T> { public T apply(T x) { return x; } }
public class BridgeUnknown extends BridgeBase<String> { @Override public String apply(String x) { return x; } public Object relay(Object x) { return apply((String)x); } }
