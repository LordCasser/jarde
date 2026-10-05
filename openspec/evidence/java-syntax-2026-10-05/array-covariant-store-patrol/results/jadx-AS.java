package defpackage;

/* JADX INFO: loaded from: AS.class */
public class AS {
    /* JADX WARN: Multi-variable type inference failed */
    static java.lang.String storeWrong() {
        new java.lang.String[2][0] = 1;
        return "unreachable";
    }

    static java.lang.String storeRight() {
        java.lang.String[] strArr = new java.lang.String[2];
        strArr[0] = "s";
        return strArr[0];
    }

    /* JADX WARN: Multi-variable type inference failed */
    static java.lang.String storeNumber() {
        new java.lang.Integer[2][0] = java.lang.Double.valueOf(2.5d);
        return "unreachable2";
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(storeRight());
        try {
            java.lang.System.out.println(storeWrong());
        } catch (java.lang.ArrayStoreException e) {
            java.lang.System.out.println("ASE1");
        }
        try {
            java.lang.System.out.println(storeNumber());
        } catch (java.lang.ArrayStoreException e2) {
            java.lang.System.out.println("ASE2");
        }
    }
}
