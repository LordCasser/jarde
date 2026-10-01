public class X2 {
    static String fmt(Exception e) { return e.getMessage(); }
    public static String single() { return fmt(new Exception("solo")); }
    public static String nested() { return fmt(new Exception("outer", new Exception("inner"))); }
    public static String plainNested() { return String.valueOf(new Object());
    }
    public static String nestedNew() { return String.valueOf(new StringBuilder("sb")); }
    public static void main(String[] a) { System.out.println(single()); System.out.println(nested()); System.out.println(nestedNew()); }
}
