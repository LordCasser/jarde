package defpackage;

/* JADX INFO: loaded from: CatchCallMarker.jar:CatchCallMarker.class */
public class CatchCallMarker<T> {
    public T value;

    public CatchCallMarker(T t, boolean z) {
        try {
            this.value = maybe(t, z);
        } catch (RuntimeException e) {
            this.value = null;
        }
    }

    private T maybe(T t, boolean z) {
        if (z) {
            throw new RuntimeException();
        }
        return t;
    }
}
