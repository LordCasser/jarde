import java.util.function.Supplier;

final class ObjectArrayForeach {
    static int sumHash(Object[] values) {
        int sum = 0;
        for (Object value : values) {
            sum += value.hashCode();
        }
        return sum;
    }

    static int sumHashFrom(Supplier<Object[]> source) {
        int sum = 0;
        for (Object value : source.get()) {
            sum += value.hashCode();
        }
        return sum;
    }
}
