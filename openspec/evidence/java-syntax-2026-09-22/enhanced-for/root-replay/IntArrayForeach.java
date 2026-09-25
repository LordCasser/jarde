import java.util.function.Supplier;

final class IntArrayForeach {
    static int sum(int[] values) {
        int sum = 0;
        for (int value : values) {
            sum += value;
        }
        return sum;
    }

    static int sumFrom(Supplier<int[]> source) {
        int sum = 0;
        for (int value : source.get()) {
            sum += value;
        }
        return sum;
    }
}
