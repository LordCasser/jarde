package defpackage;

import java.lang.Number;
import java.lang.Runnable;

/* JADX INFO: loaded from: CompatibleIntersectionBinder.jar:CompatibleIntersectionBinder.class */
public class CompatibleIntersectionBinder<T extends Number & Runnable> {
    public <U extends Number & Runnable> U identity(U x) {
        return x;
    }

    public T relay(T t) {
        return (T) identity(t);
    }
}
