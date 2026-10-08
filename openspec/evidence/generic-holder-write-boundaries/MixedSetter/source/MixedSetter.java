public class MixedSetter<T> { public T v; public MixedSetter() {} public void putT(T value) { this.v=value; } public void putObject(Object value) { this.v=(T)value; } }
