package nested;

import java.util.Objects;
import nested.SimpleOuter.Inner;

/* JADX INFO: loaded from: simple.jar:nested/UseInner.class */
public final class UseInner {
    public static Object make(SimpleOuter simpleOuter, int i) {
        Objects.requireNonNull(simpleOuter);
        return simpleOuter.new Inner(SimpleOuter.mark("A", i));
    }
}
