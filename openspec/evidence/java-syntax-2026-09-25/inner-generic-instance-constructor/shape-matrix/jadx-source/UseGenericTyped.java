package matrix;

import java.util.Objects;

/* JADX INFO: loaded from: original-input-g-none.jar:matrix/UseGenericTyped.class */
public final class UseGenericTyped {
    public static Outer.A<String>.Generic<Integer> make(Outer.A<String> a, int i) {
        Objects.requireNonNull(a);
        return new Outer.A.Generic<>(Integer.valueOf(Outer.A.mark(i)));
    }
}
