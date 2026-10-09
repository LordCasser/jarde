public class MethodHandleUse<T> { public T identity(T x) { return x; } public T relay(T x) { java.util.function.Function<T,T> f=this::identity; return f.apply(x); } }
