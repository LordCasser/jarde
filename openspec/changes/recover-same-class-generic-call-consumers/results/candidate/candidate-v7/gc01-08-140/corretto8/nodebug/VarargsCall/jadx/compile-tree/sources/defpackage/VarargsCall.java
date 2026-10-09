package defpackage;

/* JADX INFO: loaded from: VarargsCall.jar:VarargsCall.class */
public class VarargsCall<T> {
    @SafeVarargs
    public final <U> U first(U... uArr) {
        return uArr[0];
    }

    public T relay(T t) {
        return (T) first(t);
    }
}
