package defpackage;

/* JADX INFO: loaded from: ReverseDeclarationRelay.jar:ReverseDeclarationRelay.class */
public class ReverseDeclarationRelay<T> {
    public T identity(T t) {
        return t;
    }

    public T relay2(T t) {
        return identity(t);
    }

    public T relay1(T t) {
        return relay2(t);
    }

    public T relay0(T t) {
        return relay1(t);
    }
}
