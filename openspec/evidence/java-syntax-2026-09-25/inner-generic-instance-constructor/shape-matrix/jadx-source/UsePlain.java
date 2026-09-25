package matrix;

import java.util.Objects;

/* JADX INFO: loaded from: original-input-g-none.jar:matrix/UsePlain.class */
public final class UsePlain {
    public static Object make(Outer.A<String> a, int i) {
        Objects.requireNonNull(a);
        return new Outer.A.Plain(Outer.A.mark(i));
    }
}
