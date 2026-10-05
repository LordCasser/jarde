import java.util.*;
public class OB {
    static boolean eq(String a, String b){ return Objects.equals(a, b); }                  // null 安全 equals
    static int h(String a){ return Objects.hashCode(a); }
    static int hh(String a){ return a == null ? 0 : a.hashCode(); }                        // 手写版对照
    static String str(Object o){ return Objects.toString(o, "dflt"); }                     // toString 带默认
    static boolean deepEq(int[][] x, int[][] y){ return Objects.deepEquals(x, y); }        // deepEquals
    static String reqFmt(String s){ return Objects.requireNonNull(s, () -> "blank at " + System.nanoTime()).trim(); }  // requireNonNull+supplier
    enum Flag { A, B, C }
    static String flags(EnumSet<Flag> fs){ EnumSet<Flag> both = EnumSet.of(Flag.A, Flag.C); both.retainAll(fs); return both.contains(Flag.A) ? "hasA" : "no"; }  // EnumSet
    public static void main(String[] a){ System.out.println(""+eq(null,null)+"/"+eq("x",null)+"/"+h(null)+"/"+hh(null)+"/"+str(null)+"/"+deepEq(new int[][]{{1}}, new int[][]{{1}})+"/"+flags(EnumSet.of(Flag.A, Flag.B))+"/"+flags(EnumSet.noneOf(Flag.class))); }
}
