public class HistoricalControlFlow {
    public int add(int left, int right) {
        return left + right;
    }

    public int finallyPath(int value) {
        try {
            return value + 1;
        } finally {
            value = value + 2;
        }
    }
}
