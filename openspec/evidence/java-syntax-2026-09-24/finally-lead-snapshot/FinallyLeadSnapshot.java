public class FinallyLeadSnapshot {
    public static int value;

    public static int run() {
        value = 41;
        try {
            return value;
        } finally {
            value = 99;
        }
    }
}
