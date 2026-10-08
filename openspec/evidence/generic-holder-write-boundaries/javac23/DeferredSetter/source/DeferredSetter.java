public class DeferredSetter<T> { public T v; public DeferredSetter() {} public void put(T x) { sink(x); this.v=x; } private void sink(T x) {} }
