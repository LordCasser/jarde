package defpackage;

/* JADX INFO: loaded from: RawNewHold.jar:RawNewHold.class */
public class RawNewHold<T> {
    public T v;

    public RawNewHold(T v) {
        this.v = v;
    }

    public static RawNewHold make(Object value) {
        return new RawNewHold(value);
    }
}
