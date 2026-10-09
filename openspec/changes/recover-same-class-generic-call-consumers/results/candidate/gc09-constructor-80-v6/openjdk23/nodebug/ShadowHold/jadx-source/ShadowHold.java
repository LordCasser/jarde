/* JADX INFO: loaded from: ShadowHold.jar:ShadowHold.class */
public class ShadowHold<T> {
    public T v;

    /* JADX WARN: Multi-variable type inference failed */
    private T shadow(Object obj) {
        return obj;
    }

    public <T> ShadowHold(T t) {
        this.v = shadow(t);
    }
}
