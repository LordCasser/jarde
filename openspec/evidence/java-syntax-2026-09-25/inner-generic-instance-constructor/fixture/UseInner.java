package nested;

public final class UseInner {
    public static Object make(SimpleOuter outer, int value) {
        return outer.new Inner(SimpleOuter.mark("A", value));
    }
}
