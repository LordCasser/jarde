package defpackage;

/* JADX INFO: loaded from: MixedSetter.jar:MixedSetter.class */
public class MixedSetter<T> {
    public T v;

    public void putT(T t) {
        this.v = t;
    }

    /* JADX WARN: Multi-variable type inference failed */
    public void putObject(Object obj) {
        this.v = obj;
    }
}
