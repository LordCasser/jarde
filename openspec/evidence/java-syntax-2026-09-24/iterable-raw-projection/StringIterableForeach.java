import java.util.function.Supplier;

final class StringIterableForeach {
    static int sumLengths(Iterable values) {
        int sum = 0;
        for (Object item : values) {
            String value = (String) item;
            sum += value.length();
        }
        return sum;
    }

    static int sumLengthsFrom(Supplier source) {
        int sum = 0;
        for (Object item : (Iterable) source.get()) {
            String value = (String) item;
            sum += value.length();
        }
        return sum;
    }
}
