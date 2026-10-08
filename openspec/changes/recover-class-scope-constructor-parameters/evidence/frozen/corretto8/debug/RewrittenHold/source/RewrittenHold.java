public class RewrittenHold<T> { public T v; @SuppressWarnings("unchecked") public RewrittenHold(T v) { v=(T)String.valueOf(v); this.v=v; } }
