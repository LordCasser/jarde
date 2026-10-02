package defpackage;

/* JADX INFO: loaded from: DWVariants.class */
public class DWVariants {
    public static String throwInDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            if (i % 3 != 0 && i > 7) {
                throw new IllegalStateException("early" + i);
            }
        }
        return "i=" + i;
    }

    public static String throwConstDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            if (i % 3 != 0 && i > 7) {
                throw new IllegalStateException("early");
            }
        }
        return "i=" + i;
    }

    public static String doubleReturnDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            if (i % 3 != 0) {
                if (i > 7) {
                    return "a" + i;
                }
                if (i > 8) {
                    return "b" + i;
                }
            }
        }
        return "i=" + i;
    }

    public static String sharedLeafDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            if (i % 3 != 0 && (i > 7 || i > 8)) {
                return "early" + i;
            }
        }
        return "i=" + i;
    }

    public static void main(String[] strArr) {
        try {
            System.out.println(throwInDoWhile());
        } catch (IllegalStateException e) {
            System.out.println(e.getMessage());
        }
        try {
            System.out.println(throwConstDoWhile());
        } catch (IllegalStateException e2) {
            System.out.println(e2.getMessage());
        }
        System.out.println(doubleReturnDoWhile());
        System.out.println(sharedLeafDoWhile());
    }
}
