public class CompatibleIntersectionBinder<T extends Number & Runnable> { public <U extends Number & Runnable> U identity(U x) { return x; } public T relay(T x) { return identity(x); } }
