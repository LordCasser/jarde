public class IncompleteSite<T> { public T identity(T x) { return x; } public T relay(T x, boolean use) { if (use) return identity(x); return x; } }
