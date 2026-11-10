public final class CheckedReference {
    public static Integer cast() { Object value = "text"; return (Integer) value; }
    public static boolean test() { Object value = "text"; return value instanceof Integer; }
}
