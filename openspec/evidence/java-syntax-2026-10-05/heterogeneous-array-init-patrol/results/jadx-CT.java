package defpackage;

/* JADX INFO: loaded from: CT.class */
public class CT {
    static java.lang.String io(java.lang.Object obj) {
        return obj instanceof java.lang.String ? ((java.lang.String) obj).toUpperCase() : "?";
    }

    static java.lang.String io2(java.lang.Object obj) {
        return obj instanceof java.lang.String ? ((java.lang.String) obj).trim() : "?";
    }

    static java.lang.Object up() {
        return java.util.Arrays.asList("a", "b");
    }

    static java.lang.Number cov() {
        return (java.lang.Number) java.util.Arrays.asList(1, 2L).get(0);
    }

    static int prim() {
        java.lang.Integer num = 3;
        return java.lang.Double.valueOf(num.intValue()).intValue();
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(io("x") + "/" + io2(" y ") + "/" + up() + "/" + cov() + "/" + prim());
    }
}
