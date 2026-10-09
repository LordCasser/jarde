/* JADX INFO: loaded from: RawNewHold.jar:RawNewHold.class */
public class RawNewHold<T> {
    public T v;

    public RawNewHold(T t) {
        this.v = t;
    }

    public static RawNewHold make(Object obj) {
        return new RawNewHold(obj);
    }
}
