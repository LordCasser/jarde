public class NullCall<T> { public T identity(T x) { return x; } public T relay() { return identity(null); } }
