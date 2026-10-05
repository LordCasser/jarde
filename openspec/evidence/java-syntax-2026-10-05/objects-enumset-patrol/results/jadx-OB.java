package defpackage;

/* JADX INFO: loaded from: OB.class */
public class OB {

    /* JADX INFO: loaded from: OB$Flag.class */
    enum Flag {
        A,
        B,
        C
    }

    static boolean eq(java.lang.String str, java.lang.String str2) {
        return java.util.Objects.equals(str, str2);
    }

    static int h(java.lang.String str) {
        return java.util.Objects.hashCode(str);
    }

    static int hh(java.lang.String str) {
        if (str == null) {
            return 0;
        }
        return str.hashCode();
    }

    static java.lang.String str(java.lang.Object obj) {
        return java.util.Objects.toString(obj, "dflt");
    }

    static boolean deepEq(int[][] iArr, int[][] iArr2) {
        return java.util.Objects.deepEquals(iArr, iArr2);
    }

    static java.lang.String reqFmt(java.lang.String str) {
        return ((java.lang.String) java.util.Objects.requireNonNull(str, (java.util.function.Supplier<java.lang.String>) () -> {
            return "blank at " + java.lang.System.nanoTime();
        })).trim();
    }

    static java.lang.String flags(java.util.EnumSet<OB.Flag> enumSet) {
        java.util.EnumSet enumSetOf = java.util.EnumSet.of(OB.Flag.A, OB.Flag.C);
        enumSetOf.retainAll(enumSet);
        return enumSetOf.contains(OB.Flag.A) ? "hasA" : "no";
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + eq(null, null) + "/" + eq("x", null) + "/" + h(null) + "/" + hh(null) + "/" + str(null) + "/" + deepEq(new int[][]{new int[]{1}}, new int[][]{new int[]{1}}) + "/" + flags(java.util.EnumSet.of(OB.Flag.A, OB.Flag.B)) + "/" + flags(java.util.EnumSet.noneOf(OB.Flag.class)));
    }
}
