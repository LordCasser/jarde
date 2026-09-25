public class SynchronizedMultiExit {
    public static int trace;
    public static int failId;
    public static final RuntimeException FAILURE = new IllegalStateException("producer");

    private static int produce(int id) {
        trace = trace * 10 + id;
        if (failId == id) {
            throw FAILURE;
        }
        return id * 10;
    }

    public static int choose(Object lock, boolean first) {
        synchronized (lock) {
            if (first) {
                return produce(1);
            }
            return produce(2);
        }
    }
}
