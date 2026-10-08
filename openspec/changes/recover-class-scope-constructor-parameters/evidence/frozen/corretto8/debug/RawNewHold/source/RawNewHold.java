public class RawNewHold<T> { public T v; public RawNewHold(T v) { this.v = v; } @SuppressWarnings("rawtypes") public static RawNewHold make(Object value) { return new RawNewHold(value); } }
