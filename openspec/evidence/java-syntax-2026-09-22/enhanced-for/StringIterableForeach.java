import java.util.function.Supplier;

final class StringIterableForeach {
    static int sumLengths(Iterable<String> values) {
        int sum = 0;
        for (String value : values) {
            sum += value.length();
        }
        return sum;
    }

    static int sumLengthsFrom(Supplier<Iterable<String>> source) {
        int sum = 0;
        for (String value : source.get()) {
            sum += value.length();
        }
        return sum;
    }
}
