package nested;

import java.util.Objects;

/* JADX INFO: loaded from: missing-target.jar:nested/UseInner.class */
public final class UseInner {
    public static Object make(SimpleOuter simpleOuter, int i) {
        Objects.requireNonNull(simpleOuter);
        return new SimpleOuter.Inner(simpleOuter, SimpleOuter.mark("A", i));
    }
}
