public class BV extends java.lang.Object {
    public BV() {
        super();
        return;
    }

    static boolean all(boolean arg0, boolean arg1) {
        boolean local2 = arg0;
        local2 = local2 & arg1;
        return local2;
    }

    static boolean any(boolean arg0, boolean arg1) {
        boolean local2 = arg0;
        local2 = local2 | arg1;
        return local2;
    }

    static boolean flip(boolean arg0) {
        boolean local1 = arg0;
        local1 = !local1;
        return local1;
    }

    @java.lang.SafeVarargs
    static int count(java.lang.Object... arg0) {
        int local1;
        java.lang.Object[] local2;
        local1 = 0;
        local2 = arg0;
        for (java.lang.Object local5 : local2) {
            if (local5 != null) {
                local1 = local1 + 1;
            }
        }
        return local1;
    }

    static java.lang.String kind(char arg0) {
        switch (arg0) {
            case 'a':
                return "A";
            case 'b':
            case 'c':
                return "BC";
            default:
                return "?";
        }
    }

    public static void main(java.lang.String[] arg0) {
        java.io.PrintStream saved0 = java.lang.System.out;
        java.lang.StringBuilder saved1 = new java.lang.StringBuilder().append("").append(all(true, false)).append("/").append(all(true, true)).append("/").append(any(false, true)).append("/").append(flip(true)).append("/");
        saved0.println((java.lang.String) saved1.append(count((java.lang.Object[]) new java.lang.String[]{"x", null, "y"})).append("/").append((java.lang.String) kind('a')).append((java.lang.String) kind('b')).append((java.lang.String) kind('z')).toString());
        return;
    }
}
