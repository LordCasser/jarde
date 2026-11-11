public class CallRelay<T> { public T identity(T x) { return x; } public T relay(T x) { return identity(x); } }
