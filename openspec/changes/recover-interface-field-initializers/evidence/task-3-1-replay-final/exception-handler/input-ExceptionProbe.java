public class ExceptionProbe {
    public static final String FIRST = BoundaryEffects.first();
    public static final String SECOND = BoundaryEffects.second();
    static {
        try { BoundaryEffects.fail(); }
        catch (IllegalStateException expected) { BoundaryEffects.caught(); }
    }
    public static String observe() { return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND; }
}
