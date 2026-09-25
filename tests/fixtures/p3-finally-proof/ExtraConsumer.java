public class ExtraConsumer {
    static int trace;

    static int cleanup() { return 2; }

    public static int run() {
        try {
            return trace;
        } finally {
            int value = cleanup();
            trace += value;
        }
    }
}
