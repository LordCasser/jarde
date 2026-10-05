package defpackage;

/* JADX INFO: loaded from: PS.class */
public class PS {
    /* JADX WARN: Can't fix incorrect switch cases order, some code will duplicate */
    static java.lang.String scan(java.lang.String str) {
        java.lang.StringBuilder sb = new java.lang.StringBuilder();
        int i = 0;
        int i2 = 0;
        char c = 0;
        for (int i3 = 0; i3 < str.length(); i3++) {
            char cCharAt = str.charAt(i3);
            switch (cCharAt) {
                case '\t':
                case ' ':
                    c = cCharAt;
                    break;
                case '0':
                case '1':
                case '2':
                case '3':
                    i2++;
                    c = cCharAt;
                    break;
                default:
                    if (cCharAt != c) {
                        sb.append(java.lang.Character.toUpperCase(cCharAt));
                        i++;
                        c = cCharAt;
                    }
                    break;
            }
        }
        return sb.toString() + "/" + i + "/" + i2;
    }

    static int sumSquares(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            i += i2 * i2;
        }
        return i;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(scan("a b\t112 a z") + "/" + sumSquares(new int[]{2, 3}));
    }
}
