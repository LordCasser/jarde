public class VoidDirect<T> { public Object seen; public int calls; public void sink(T x) { seen=x; calls++; } public void relay(T x) { sink(x); } }
