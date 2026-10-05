public class AR extends java.lang.Object {
    public AR() {
        super();
        return;
    }

    static int lenInt(java.lang.Object arg0) {
        if (arg0 instanceof int[]) {
            return ((int[]) arg0).length;
        } else {
            return -1;
        }
    }

    static java.lang.String firstStr(java.lang.Object arg0) {
        if (arg0 instanceof java.lang.String[]) {
            return ((java.lang.String[]) arg0)[0];
        } else {
            return "-";
        }
    }

    static java.lang.String kind(java.lang.Object arg0) {
        if (arg0 instanceof int[]) {
            return "int[]";
        } else if (arg0 instanceof java.lang.String[]) {
            return "String[]";
    } else if (arg0 instanceof java.lang.Object[]) {
            return "Object[]";
    } else if (arg0 instanceof java.lang.String) {
            return "String";
    } else {
            return "other";
    }
    }

    static java.lang.Object covariantUp(int[] arg0) {
        return arg0;
    }

    public static void main(java.lang.String[] arg0) {
        java.io.PrintStream saved0 = java.lang.System.out;
        java.lang.StringBuilder saved1 = new java.lang.StringBuilder().append("");
        java.lang.StringBuilder saved2 = saved1.append(lenInt((java.lang.Object) new int[]{1, 2, 3})).append("/").append(lenInt((java.lang.Object) "s")).append("/");
        java.lang.StringBuilder saved3 = saved2.append((java.lang.String) firstStr((java.lang.Object) new java.lang.String[]{"x"})).append("/").append((java.lang.String) kind((java.lang.Object) new java.lang.Object[1])).append("/").append((java.lang.String) kind((java.lang.Object) java.lang.Integer.valueOf(5))).append("/");
        saved0.println((java.lang.String) saved3.append(covariantUp(new int[]{7}) instanceof int[]).toString());
        return;
    }
}
