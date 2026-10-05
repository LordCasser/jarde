public class PM extends java.lang.Object {
    static final int READ = 1;

    static final int WRITE = 2;

    static final int EXEC = 4;

    static final int ADMIN = 8;

    public PM() {
        super();
        return;
    }

    static int grant(int arg0, int arg1) {
        return arg0 | arg1;
    }

    static int revoke(int arg0, int arg1) {
        return arg0 & (arg1 ^ -1);
    }

    static boolean can(int arg0, int arg1) {
        return (arg0 & arg1) != 0;
    }

    static boolean canAll(int arg0, int arg1) {
        return (arg0 & arg1) == arg1;
    }

    static int fromOrdinal(int arg0) {
        return 1 << arg0;
    }

    public static void main(java.lang.String[] arg0) {
        int local1 = 0;
        local1 = grant(local1, 5);
        local1 = grant(local1, 2);
        local1 = revoke(local1, 4);
        java.lang.System.out.println("" + can(local1, 1) + "/" + can(local1, 4) + "/" + canAll(local1, 3) + "/" + fromOrdinal(3) + "/" + local1);
        return;
    }
}
