public class TypedReceiverRelay<T> { public T identity(T x) { return x; } public T relay(TypedReceiverRelay<T> receiver, T x) { return receiver.identity(x); } }
