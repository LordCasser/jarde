public class V17a implements AutoCloseable {
    public static boolean boom = false;
    @Override public void close() { if (boom) { throw new IllegalStateException("close"); } }
    static void touch(V17a r) { if (boom) { throw new IllegalStateException("touch"); } System.out.println("touched"); }
    static String give() { if (boom) { throw new IllegalStateException("give"); } return "g"; }
    interface G17 { String get(); }
    static final class G17Impl implements G17 {
        @Override public String get() { return "ig"; }
    }
    static G17 pick() { if (boom) { throw new IllegalStateException("pick"); } return new G17Impl(); }
    public static String pureCalls() throws Exception {
        try (V17a r = new V17a()) { r.toString(); r.hashCode(); }
        return "done";
    }
    public static String mixedVoidAndCall() throws Exception {
        try (V17a r = new V17a()) { touch(r); r.toString(); }
        return "done";
    }
    public static String callBeforeReturnInside() throws Exception {
        try (V17a r = new V17a()) { touch(r); r.hashCode(); return "in"; }
    }
    public static String callOnlyReturnInside() throws Exception {
        try (V17a r = new V17a()) { r.toString(); return "solo"; }
    }
    public static String staticCall() throws Exception {
        try (V17a r = new V17a()) { give(); }
        return "done";
    }
    public static String virtualCall() throws Exception {
        try (V17a r = new V17a()) { r.toString(); }
        return "done";
    }
    public static String interfaceCall() throws Exception {
        try (V17a r = new V17a()) { pick().get(); }
        return "done";
    }
    public static String receivedLocal() throws Exception {
        try (V17a r = new V17a()) { String s = r.toString(); }
        return "done";
    }
    public static void main(String[] a) throws Exception {
        System.out.println(pureCalls());
        System.out.println(mixedVoidAndCall());
        System.out.println(callBeforeReturnInside());
        System.out.println(callOnlyReturnInside());
        System.out.println(staticCall());
        System.out.println(virtualCall());
        System.out.println(interfaceCall());
        System.out.println(receivedLocal());
    }
}
