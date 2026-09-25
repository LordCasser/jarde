package nested;

public final class IdentityUse {
    public static Object make(SimpleOuter checked, SimpleOuter actual, int value) {
        return actual.new Inner(SimpleOuter.mark("A", value));
    }
}
