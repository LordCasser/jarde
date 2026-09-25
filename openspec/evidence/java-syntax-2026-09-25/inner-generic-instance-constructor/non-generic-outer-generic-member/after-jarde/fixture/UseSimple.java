package minimal;
public final class UseSimple {
    public static Object make(Outer outer, int value) {
        return outer.new Inner<>(Integer.valueOf(value));
    }
}
