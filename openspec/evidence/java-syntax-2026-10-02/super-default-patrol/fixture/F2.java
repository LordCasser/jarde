public class F2 {
    static int value;
    static {
        if (Boolean.getBoolean("boom")) { throw new IllegalStateException("clinit-fail"); }
        value = 5;
    }
    static int other = init();
    static int init() {
        try { return Integer.parseInt("42"); }
        catch (NumberFormatException e) { throw new RuntimeException("wrap", e); }
    }
    public static void main(String[] a) { System.out.println(value + ":" + other); }
}
