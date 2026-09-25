public class CastProbe {
    public static String direct(Object x) { return (String) x; }
    public static int receiver(Object x) { return ((String) x).length(); }
    public static int array(Object x, int i) { return ((int[]) x)[i]; }
    public static String[] referenceArray(Object x) { return (String[]) x; }
    public static Object widen(Object x) { String s = (String) x; return s; }
    public static String twice(Object x) { return (String) (CharSequence) x; }
    public static String invocation(java.util.function.Supplier<Object> f) { return (String) f.get(); }
    public static int catches(Object x) {
        try { return ((String) x).length(); }
        catch (ClassCastException e) { return 7; }
    }
}
