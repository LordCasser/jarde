package matrix;

import java.util.Objects;

/* JADX INFO: loaded from: original-input-g-none.jar:matrix/UseGenericObject.class */
public final class UseGenericObject {
    public static Object make(Outer.A<String> a, int i) {
        Objects.requireNonNull(a);
        return new Outer.A.Generic(Integer.valueOf(Outer.A.mark(i)));
    }
}
