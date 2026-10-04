package defpackage;

/* JADX INFO: loaded from: SW.class */
public class SW {
    static java.lang.String s(int i) {
        switch (i) {
            case 1:
                return "one";
            case 2:
                return "two";
            default:
                return "other";
        }
    }

    static int t(int i) {
        int i2;
        switch (i) {
            case 1:
                i2 = 10;
                break;
            case 2:
                i2 = 20;
                break;
            default:
                i2 = 0;
                break;
        }
        return i2;
    }

    static java.lang.String str(java.lang.String str) {
        switch (str) {
            case "a":
                return "A";
            case "b":
            case "c":
                return "BC";
            default:
                return "?";
        }
    }

    static int fal(int i) {
        switch (i) {
            case 1:
            case 2:
                return 12;
            case 3:
                return i * 2;
            default:
                return 0;
        }
    }

    static int yld(int i) {
        return switch8(i);
    }

    static int switch8(int i) {
        return i;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(s(1) + "/" + t(2) + "/" + str("b") + "/" + fal(3));
    }
}
