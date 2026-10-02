package defpackage;

/* JADX INFO: loaded from: L5.class */
public class L5 {
    public static String brkSelfContSelf(int i) {
        StringBuilder sb = new StringBuilder();
        for (int i2 = 0; i2 < i; i2++) {
            for (int i3 = 0; i3 < i; i3++) {
                for (int i4 = 0; i4 < i && i4 != 1; i4++) {
                    if (i4 != 0) {
                        sb.append(i2).append(i3).append(i4).append(' ');
                    }
                }
            }
        }
        return sb.toString();
    }

    public static String brkSelfContMid(int i) {
        StringBuilder sb = new StringBuilder();
        for (int i2 = 0; i2 < i; i2++) {
            for (int i3 = 0; i3 < i; i3++) {
                for (int i4 = 0; i4 < i && i4 != 1; i4++) {
                    if (i3 != 2) {
                        sb.append(i2).append(i3).append(i4).append(' ');
                    }
                }
            }
        }
        return sb.toString();
    }

    public static String dblJumpDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            if (i % 3 != 0 && i > 7) {
                break;
            }
        }
        return "i=" + i;
    }

    public static String dblJumpDoWhilePlain() {
        int i = 0;
        while (i < 10) {
            i++;
            if (i % 3 != 0 && i > 7) {
                return "early";
            }
        }
        return "i=" + i;
    }

    public static void main(String[] strArr) {
        System.out.println(brkSelfContSelf(2));
        System.out.println(brkSelfContMid(3));
        System.out.println(dblJumpDoWhile());
        System.out.println(dblJumpDoWhilePlain());
    }
}
