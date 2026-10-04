public class BA extends java.lang.Object {
    private boolean flag;

    private int count;

    public BA() {
        super();
        this.flag = false;
        this.count = 0;
        return;
    }

    public static void main(java.lang.String[] arg0) {
        BA local1 = new BA();
        local1.new S().setB(true);
        local1.new S().setI(7);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(local1.flag).append("/").append(local1.count).toString());
        return;
    }

    static boolean access$002(BA arg0, boolean arg1) {
        arg0.flag = arg1;
        return arg1;
    }

    static int access$102(BA arg0, int arg1) {
    }

    class S extends java.lang.Object {
        S() {
            super();
            return;
        }

        void setB(boolean arg1) {
            BA.access$002(BA.this, arg1);
            return;
        }

        void setI(int arg1) {
            BA.access$102(BA.this, arg1);
            return;
        }
    }
}
