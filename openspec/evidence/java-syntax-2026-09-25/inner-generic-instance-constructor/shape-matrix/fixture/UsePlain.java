package matrix;

public final class UsePlain {
    public static Object make(Outer.A<String> outer, int value) {
        return outer.new Plain(Outer.A.mark(value));
    }
}
