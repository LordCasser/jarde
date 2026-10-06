public class ENN {
    enum Flag { A, B, C }                                                                   // 巡查锚的 `$` 嵌套形（池形名，未折叠）
    static String flags(java.util.EnumSet<Flag> fs){ java.util.EnumSet<Flag> both = java.util.EnumSet.of(Flag.A, Flag.C); both.retainAll(fs); return both.contains(Flag.A) ? "hasA" : "no"; }
    public static void main(String[] a){ System.out.println(flags(java.util.EnumSet.of(Flag.A, Flag.B))); }
}
