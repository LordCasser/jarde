public class ExceptionHold<T> { public T v; public ExceptionHold(T v) { try { this.v=maybe(v); } catch (RuntimeException ex) { this.v=null; } } private T maybe(T x) { return x; } }
