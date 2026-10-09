public class NumberBoundRelay<T extends Number> { public T identity(T x) { return x; } public T relay(T x) { return identity(x); } }
