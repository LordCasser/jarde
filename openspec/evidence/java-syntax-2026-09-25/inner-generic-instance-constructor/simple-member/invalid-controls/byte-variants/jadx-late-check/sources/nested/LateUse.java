package nested;

import java.util.Objects;
import nested.SimpleOuter.Inner;

/* JADX INFO: loaded from: late-check.jar:nested/LateUse.class */
public final class LateUse {
    public static Object make(SimpleOuter simpleOuter, int i) {
        int iMark = SimpleOuter.mark("A", i);
        Objects.requireNonNull(simpleOuter);
        return simpleOuter.new Inner(iMark);
    }
}
