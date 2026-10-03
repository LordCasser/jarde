// Variant 3 (static and default together): one interface child whose static method is called from
// the root and whose default method reaches only the child's own presentation.
public class WCDefault {
    interface A { static int sv() { return 8; } default int dv() { return 3; } }
    static class Use implements A { }
    public static void main(String[] a) { System.out.println(A.sv() + new Use().dv()); }
}
