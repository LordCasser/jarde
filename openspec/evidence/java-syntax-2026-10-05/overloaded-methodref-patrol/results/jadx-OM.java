package defpackage;

/* JADX INFO: loaded from: OM.class */
public class OM {
    static java.lang.String pickStr(java.util.function.Function<java.lang.Integer, java.lang.String> function) {
        return function.apply(7);
    }

    static int pickInt(java.util.function.Function<java.lang.String, java.lang.Integer> function) {
        return function.apply("42").intValue();
    }

    static java.lang.Integer pickIdent(java.util.function.Function<java.lang.Integer, java.lang.Integer> function) {
        return function.apply(9);
    }

    static java.lang.String overloaded(java.util.function.Function<java.lang.Integer, java.lang.String> function) {
        return function.apply(3);
    }

    static java.lang.String o(int i) {
        return "i" + i;
    }

    static java.lang.String o(java.lang.String str) {
        return "s" + str;
    }

    static java.lang.String o(java.lang.Object obj) {
        return "o" + obj;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(pickStr((v0) -> {
            return java.lang.String.valueOf(v0);
        }) + "/" + pickInt(java.lang.Integer::valueOf) + "/" + pickIdent((v0) -> {
            return java.lang.Integer.valueOf(v0);
        }) + "/" + overloaded((v0) -> {
            return o(v0);
        }) + "/" + overloaded(num -> {
            return "L" + num;
        }));
    }
}
