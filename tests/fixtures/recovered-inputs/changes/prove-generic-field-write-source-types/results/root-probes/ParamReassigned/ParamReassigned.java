public class ParamReassigned<T> { public T v; public void put(T x, Object y) { x=(T)y; this.v=x; } }
