package matrix;

public final class UseGenericTyped {
    public static Outer.A<String>.Generic<Integer> make(Outer.A<String> outer, int value) {
        return outer.new Generic<Integer>(Integer.valueOf(Outer.A.mark(value)));
    }
}
