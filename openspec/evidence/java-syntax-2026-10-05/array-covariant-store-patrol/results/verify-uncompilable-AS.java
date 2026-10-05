public class AS extends java.lang.Object {
    public AS() {
        super();
        return;
    }

    static java.lang.String storeWrong() {
        java.lang.String[] local0 = new java.lang.String[2];
        local0[0] = java.lang.Integer.valueOf(1);
        return "unreachable";
    }

    static java.lang.String storeRight() {
        java.lang.String[] local0 = new java.lang.String[2];
        local0[0] = "s";
        return (java.lang.String) local0[0];
    }

    static java.lang.String storeNumber() {
        java.lang.Integer[] local0 = new java.lang.Integer[2];
        local0[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);
        return "unreachable2";
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) storeRight());
        try {
            java.lang.System.out.println((java.lang.String) storeWrong());
        } catch (java.lang.ArrayStoreException local1) {
            java.lang.System.out.println("ASE1");
        }
        try {
            java.lang.System.out.println((java.lang.String) storeNumber());
        } catch (java.lang.ArrayStoreException local1) {
            java.lang.System.out.println("ASE2");
        }
        return;
    }
}
