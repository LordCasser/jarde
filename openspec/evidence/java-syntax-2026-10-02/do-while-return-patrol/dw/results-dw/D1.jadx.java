package defpackage;

/* JADX INFO: loaded from: D1.class */
public class D1 {
    static int hits;

    public static String retInDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            if (i % 3 != 0) {
                if (i > 7) {
                    return "early:" + hits;
                }
                hits++;
            }
        }
        return "i=" + i;
    }

    public static String retInPlainDo() {
        int i = 0;
        while (i != 3) {
            i++;
            if (i >= 5) {
                return "d" + i;
            }
        }
        return "d3";
    }

    public static String retInIf() {
        for (int i = 0; i < 5; i++) {
            if (i == 2) {
                return "f2";
            }
        }
        return "end";
    }

    public static void main(String[] strArr) {
        System.out.println(retInDoWhile());
        System.out.println(retInPlainDo());
        System.out.println(retInIf());
    }
}
