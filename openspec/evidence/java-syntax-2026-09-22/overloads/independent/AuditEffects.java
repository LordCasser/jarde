public class AuditEffects {
    public static int count;
    public static String mark(int digit, boolean fail) {
        count = count * 10 + digit;
        if (fail) throw new IllegalArgumentException();
        return "x";
    }
}
