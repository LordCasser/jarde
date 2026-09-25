import java.lang.reflect.Field;
public class StringSwitchUnicodeChooseRunner {
    private static final Field CALLS;
    static {
        try {
            CALLS = StringSwitchUnicode.class.getDeclaredField("calls");
            CALLS.setAccessible(true);
        } catch (ReflectiveOperationException e) { throw new ExceptionInInitializerError(e); }
    }
    private static void check(String label, String value) throws Exception {
        CALLS.setInt(null, 0);
        try {
            System.out.println(label + ":" + StringSwitchUnicode.choose(value) + ":calls=" + CALLS.getInt(null));
        } catch (RuntimeException e) {
            System.out.println(label + ":" + e.getClass().getSimpleName() + ":calls=" + CALLS.getInt(null));
        }
    }
    public static void main(String[] args) throws Exception {
        check("empty", ""); check("bmp", "雪"); check("supplementary", "𐐷");
        check("default", "other"); check("null", null);
    }
}
