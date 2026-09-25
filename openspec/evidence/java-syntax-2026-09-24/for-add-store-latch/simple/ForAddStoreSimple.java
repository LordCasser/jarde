public final class ForAddStoreSimple {
    private ForAddStoreSimple() {
    }

    public static int run(int limit, int step) {
        int sum = 0;
        for (int i = 0; i < limit; i = i + step) {
            sum += i;
        }
        return sum;
    }
}
