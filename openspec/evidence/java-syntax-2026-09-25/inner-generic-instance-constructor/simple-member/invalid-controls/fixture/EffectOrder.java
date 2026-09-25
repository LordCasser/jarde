package nested;

import java.util.Objects;

public final class EffectOrder {
    public static Object qualified(SimpleOuter outer, int value) {
        return outer.new Inner(SimpleOuter.mark("A", value));
    }

    public static Object preparedBeforeCheck(SimpleOuter outer, int value) {
        int prepared = SimpleOuter.mark("A", value);
        Objects.requireNonNull(outer);
        return outer.new Inner(prepared);
    }
}
