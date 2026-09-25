public class ThrowEffects {
    public static int calls;
    public static RuntimeException expected;
    public static RuntimeException problem() { calls++; return expected; }
    public static void mark() { calls++; }
}
