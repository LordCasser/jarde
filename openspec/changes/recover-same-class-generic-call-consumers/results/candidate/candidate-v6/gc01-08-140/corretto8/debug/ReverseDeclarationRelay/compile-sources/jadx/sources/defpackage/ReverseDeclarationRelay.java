package defpackage;

/* JADX INFO: loaded from: ReverseDeclarationRelay.jar:ReverseDeclarationRelay.class */
public class ReverseDeclarationRelay<T> {
    public T identity(T x) {
        return x;
    }

    public T relay2(T x) {
        return identity(x);
    }

    public T relay1(T x) {
        return relay2(x);
    }

    public T relay0(T x) {
        return relay1(x);
    }
}
