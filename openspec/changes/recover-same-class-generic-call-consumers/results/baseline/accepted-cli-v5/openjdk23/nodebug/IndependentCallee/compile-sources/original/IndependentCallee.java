public class IndependentCallee<T> { public <U> U identity(U x) { return x; } public T relay(T x) { return this.<T>identity(x); } }
