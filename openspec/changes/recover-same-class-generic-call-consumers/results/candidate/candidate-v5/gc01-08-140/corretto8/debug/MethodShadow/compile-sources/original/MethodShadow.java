public class MethodShadow<T> { public <T extends Number> T relay(T x) { return this.<T>identity(x); } public <U extends Number> U identity(U x) { return x; } }
