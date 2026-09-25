package matrix;

import java.util.Objects;

/* JADX INFO: loaded from: original-input-g-none.jar:matrix/UsePlainRaw.class */
public final class UsePlainRaw {
    public static Object make(Outer.A a, int i) {
        Objects.requireNonNull(a);
        return new Outer.A.Plain(Outer.A.mark(i));
    }
}
