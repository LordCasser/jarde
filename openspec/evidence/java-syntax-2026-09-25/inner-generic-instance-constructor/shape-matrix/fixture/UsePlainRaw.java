package matrix;

public final class UsePlainRaw {
    public static Object make(Outer.A outer, int value) {
        return outer.new Plain(Outer.A.mark(value));
    }
}
