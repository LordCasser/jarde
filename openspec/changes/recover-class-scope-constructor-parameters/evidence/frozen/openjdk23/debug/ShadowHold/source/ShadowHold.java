public class ShadowHold<T> { public T v; private T shadow(Object x) { return (T)x; } public <T> ShadowHold(T v) { this.v=shadow(v); } }
