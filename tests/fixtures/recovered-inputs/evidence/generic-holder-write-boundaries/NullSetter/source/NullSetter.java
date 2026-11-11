public class NullSetter<T> { public T v; public NullSetter() {} public void put(T value) { this.v=value; } public void clear() { this.v=null; } }
