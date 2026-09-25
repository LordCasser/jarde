public final class SumLengthsProbe {
    public static int sumLengths(Iterable<String> values) {
        int sum = 0;
        for (String value : values) {
            sum += value.length();
        }
        return sum;
    }
}
