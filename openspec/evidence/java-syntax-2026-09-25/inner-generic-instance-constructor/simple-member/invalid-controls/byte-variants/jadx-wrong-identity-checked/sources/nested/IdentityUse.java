package nested;

import java.util.Objects;
import nested.SimpleOuter.Inner;

/* JADX INFO: loaded from: wrong-identity-checked.jar:nested/IdentityUse.class */
public final class IdentityUse {
    public static Object make(SimpleOuter simpleOuter, SimpleOuter simpleOuter2, int i) {
        Objects.requireNonNull(simpleOuter);
        return simpleOuter2.new Inner(SimpleOuter.mark("A", i));
    }
}
