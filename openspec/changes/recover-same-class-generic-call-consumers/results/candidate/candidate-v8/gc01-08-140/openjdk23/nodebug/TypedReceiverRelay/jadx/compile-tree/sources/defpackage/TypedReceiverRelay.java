package defpackage;

/* JADX INFO: loaded from: TypedReceiverRelay.jar:TypedReceiverRelay.class */
public class TypedReceiverRelay<T> {
    public T identity(T t) {
        return t;
    }

    public T relay(TypedReceiverRelay<T> typedReceiverRelay, T t) {
        return typedReceiverRelay.identity(t);
    }
}
