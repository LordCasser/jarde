public class VarargsCall<T> { @SafeVarargs public final <U> U first(U... xs) { return xs[0]; } public T relay(T x) { return this.<T>first(x); } }
