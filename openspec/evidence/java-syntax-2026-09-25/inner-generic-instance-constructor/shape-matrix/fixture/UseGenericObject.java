package matrix;

public final class UseGenericObject {
    public static Object make(Outer.A<String> outer, int value) {
        return outer.new Generic<>(Integer.valueOf(Outer.A.mark(value)));
    }
}
