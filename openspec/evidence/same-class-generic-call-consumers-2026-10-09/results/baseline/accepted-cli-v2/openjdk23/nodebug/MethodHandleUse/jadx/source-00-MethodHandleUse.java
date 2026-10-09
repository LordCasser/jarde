package defpackage;

import java.util.function.Function;

/* JADX INFO: loaded from: MethodHandleUse.jar:MethodHandleUse.class */
public class MethodHandleUse<T> {
    public T identity(T t) {
        return t;
    }

    public T relay(T t) {
        Function function = this::identity;
        return (T) function.apply(t);
    }
}
