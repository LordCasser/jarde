public class ReceiverGrouping {
    public ReceiverGrouping() {
    }

    public static String call(String a, String b) {
        return (a + b).substring(1);
    }

    public static int length(String a, String b) {
        return (a + b).length();
    }

    public static String nested(String a, String b, String c) {
        return (a + b + c).substring(1);
    }

    public static String plain(String a) {
        return a.trim();
    }

    public static int chained(String a) {
        return a.trim().length();
    }

    public static String same(String a, String b, String c) {
        return a + b + c;
    }

    public static String argument(String a, String b) {
        return wrap(a + b);
    }

    public static String wrap(String value) {
        return value;
    }
}
