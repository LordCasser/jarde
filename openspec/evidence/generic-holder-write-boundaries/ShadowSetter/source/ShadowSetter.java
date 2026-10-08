public class ShadowSetter<T> { public T v; public ShadowSetter() {} public <T> void put(T x) { ((ShadowSetter)this).v=x; } }
