public class TP extends java.lang.Object {
    public TP() {
        super();
        return;
    }

    static java.lang.String pick(java.lang.String arg0, java.lang.String arg1) {
        return (arg0 != null ? arg0 : arg1).trim();
    }

    static int idx(int[] arg0) {
        return arg0[0];
    }

    static int chain(java.lang.String arg0, java.lang.StringBuilder arg1) {
        return (arg0 != null ? arg1.append(arg0) : arg1).length();
    }

    static java.lang.String nested(java.lang.String arg0) {
        return (arg0 != null ? arg0 : "x").isEmpty() ? "E" : "N";
    }

    public static void main(java.lang.String[] arg0) {
        java.io.PrintStream saved0 = java.lang.System.out;
        java.lang.StringBuilder saved1 = new java.lang.StringBuilder().append("").append((java.lang.String) pick((java.lang.String) null, " q ")).append("/");
        saved0.println((java.lang.String) saved1.append(idx(new int[]{7, 9})).append("/").append(chain("z", new java.lang.StringBuilder())).append("/").append((java.lang.String) nested("")).append("/").append((java.lang.String) nested((java.lang.String) null)).toString());
        return;
    }
}
