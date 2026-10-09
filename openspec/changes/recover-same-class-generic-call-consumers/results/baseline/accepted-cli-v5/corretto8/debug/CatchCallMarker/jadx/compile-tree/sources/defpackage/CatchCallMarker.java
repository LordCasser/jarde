package defpackage;

/* JADX INFO: loaded from: CatchCallMarker.jar:CatchCallMarker.class */
public class CatchCallMarker<T> {
    public T value;

    public CatchCallMarker(T x, boolean fail) {
        try {
            this.value = maybe(x, fail);
        } catch (RuntimeException e) {
            this.value = null;
        }
    }

    private T maybe(T x, boolean fail) {
        if (fail) {
            throw new RuntimeException();
        }
        return x;
    }
}
