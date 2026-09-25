package nested;

import java.util.Objects;
import nested.SimpleOuter.Inner;

/* JADX INFO: loaded from: full-target.jar:nested/EffectOrder.class */
public final class EffectOrder {
    public static Object qualified(SimpleOuter simpleOuter, int i) {
        Objects.requireNonNull(simpleOuter);
        return simpleOuter.new Inner(SimpleOuter.mark("A", i));
    }

    public static Object preparedBeforeCheck(SimpleOuter simpleOuter, int i) {
        int iMark = SimpleOuter.mark("A", i);
        Objects.requireNonNull(simpleOuter);
        Objects.requireNonNull(simpleOuter);
        return simpleOuter.new Inner(iMark);
    }
}
