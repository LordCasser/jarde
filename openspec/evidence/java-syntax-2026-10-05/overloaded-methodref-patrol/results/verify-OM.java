public class OM extends java.lang.Object {
    public OM() {
        super();
        return;
    }

    static java.lang.String pickStr(java.util.function.Function arg0) {
        return (java.lang.String) arg0.apply((java.lang.Object) java.lang.Integer.valueOf(7));
    }

    static int pickInt(java.util.function.Function arg0) {
        return ((java.lang.Integer) arg0.apply((java.lang.Object) "42")).intValue();
    }

    static java.lang.Integer pickIdent(java.util.function.Function arg0) {
        return (java.lang.Integer) arg0.apply((java.lang.Object) java.lang.Integer.valueOf(9));
    }

    static java.lang.String overloaded(java.util.function.Function arg0) {
        return (java.lang.String) arg0.apply((java.lang.Object) java.lang.Integer.valueOf(3));
    }

    static java.lang.String o(int arg0) {
        return "i" + arg0;
    }

    static java.lang.String o(java.lang.String arg0) {
        return "s" + arg0;
    }

    static java.lang.String o(java.lang.Object arg0) {
        return "o" + arg0;
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) pickStr((java.util.function.Function) ((java.lang.Object p0) -> java.lang.String.valueOf((java.lang.Object) (java.lang.Integer) p0)))).append("/").append(pickInt((java.util.function.Function) ((java.lang.Object p0_) -> java.lang.Integer.valueOf((java.lang.String) p0_)))).append("/").append((java.lang.Object) pickIdent((java.util.function.Function) ((java.lang.Object p0__) -> java.lang.Integer.valueOf((java.lang.Integer) p0__)))).append("/").append((java.lang.String) overloaded((java.util.function.Function) ((java.lang.Object p0___) -> OM.o((java.lang.Object) (java.lang.Integer) p0___)))).append("/").append((java.lang.String) overloaded((java.util.function.Function) ((java.lang.Object p0____) -> "L" + (java.lang.Integer) p0____))).toString());
        return;
    }
}
