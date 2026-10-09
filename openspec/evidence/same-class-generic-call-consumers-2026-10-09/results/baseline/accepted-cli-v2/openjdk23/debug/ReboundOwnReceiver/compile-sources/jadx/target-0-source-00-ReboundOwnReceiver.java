package defpackage;

/* JADX INFO: loaded from: ReboundOwnReceiver.jar:ReboundOwnReceiver.class */
public class ReboundOwnReceiver<T> {
    public T identity(T x) {
        return x;
    }

    public T relay(T x) {
        ReboundOwnReceiver<T> receiver = this;
        return receiver.identity(x);
    }
}
