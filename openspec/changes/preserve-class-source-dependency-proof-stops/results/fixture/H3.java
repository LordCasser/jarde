// The depth-boundary fixture: `L0` implements `Sig`, and each `Li` extends `Li-1`.  The walk
// allows eight chain edges, so `via(new L7())` proves at the cap (`L7`..`L0` plus `Sig` is eight)
// while `via(new L8())` is one edge beyond it and keeps the refusal.
public class H3 {
    interface Sig { String tag(); }
    static class L0 implements Sig { public String tag() { return "l0"; } }
    static class L1 extends L0 { }
    static class L2 extends L1 { }
    static class L3 extends L2 { }
    static class L4 extends L3 { }
    static class L5 extends L4 { }
    static class L6 extends L5 { }
    static class L7 extends L6 { }
    static class L8 extends L7 { }
    static String via(Sig s) { return "d:" + s.tag(); }
    public static void main(String[] a) {
        System.out.println(via(new L7()));
        System.out.println(via(new L8()));
    }
}
