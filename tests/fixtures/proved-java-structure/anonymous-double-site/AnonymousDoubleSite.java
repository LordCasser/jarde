abstract class DoubleBase {
    abstract int value();
}

public final class AnonymousDoubleSite {
    static DoubleBase create(boolean first) {
        if (first) {
            return new DoubleBase() {
                @Override
                int value() {
                    return 11;
                }
            };
        }
        return new DoubleBase() {
            @Override
            int value() {
                return 22;
            }
        };
    }

    public static void main(String[] args) {
        DoubleBase first = create(true);
        DoubleBase second = create(false);
        System.out.println("first=" + first.value());
        System.out.println("second=" + second.value());
        System.out.println("sameClass=" + (first.getClass() == second.getClass()));
    }
}
