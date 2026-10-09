public class RawOwnReceiver<T> { public T identity(T x) { return x; } @SuppressWarnings({"rawtypes","unchecked"}) public T relay(T x) { RawOwnReceiver raw=this; return (T)raw.identity(x); } }
