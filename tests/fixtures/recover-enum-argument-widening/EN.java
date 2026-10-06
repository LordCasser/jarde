public class EN {
    static String flags(java.util.EnumSet<ENT> fs){ java.util.EnumSet<ENT> both = java.util.EnumSet.of(ENT.A, ENT.C); both.retainAll(fs); return both.contains(ENT.A) ? "hasA" : "no"; }   // OB 锚（+ Collection 伴行）
    static String three(java.util.EnumSet<ENT> fs){ java.util.EnumSet<ENT> all = java.util.EnumSet.of(ENT.A, ENT.B, ENT.C); all.retainAll(fs); return all.contains(ENT.B) ? "hasB" : "no"; }  // EnumSet.of 变长形
    static <E extends java.lang.Enum<E>> java.util.EnumSet<E> same(E a, E b){ return java.util.EnumSet.of(a, b); }   // 真 Enum 型实参（同型，不引入 cast）
    static boolean eq(String a, String b){ return java.util.Objects.equals(a, b); }        // 无枚举路径零漂移对照
    static int h(String a){ return java.util.Objects.hashCode(a); }
    static String str(Object o){ return java.util.Objects.toString(o, "dflt"); }
    public static void main(String[] a){ System.out.println(flags(java.util.EnumSet.of(ENT.A, ENT.B))+"/"+three(java.util.EnumSet.noneOf(ENT.class))+"/"+same(ENT.A, ENT.B).contains(ENT.A)+"/"+eq(null,null)+"/"+h(null)+"/"+str(null)); }
}
