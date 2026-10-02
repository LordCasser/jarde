package defpackage;

/* JADX INFO: loaded from: fam.jar:V1.class */
public class V1 {

    /* JADX INFO: loaded from: fam.jar:V1$Op.class */
    enum Op {
        ADD,
        SUB,
        MUL
    }

    public static int numSwitch(int i, int i2) {
        switch (i) {
            case 1:
                return i2 + 1;
            case 2:
                return i2 * 2;
            case 3:
            case 4:
            default:
                return -i2;
            case 5:
                return i2 - 5;
        }
    }

    public static String strSwitch(String str) {
        switch (str) {
            case "alpha":
                return "A";
            case "beta":
                return "B";
            case "gamma":
                return "G";
            default:
                return "?";
        }
    }

    public static int enumSwitch(Op op, int i) {
        switch (op) {
            case ADD:
                return i + 1;
            case SUB:
                return i - 1;
            case MUL:
                return i * 2;
            default:
                return 0;
        }
    }

    public static int fallThrough(int i) {
        int i2;
        int i3 = 0;
        switch (i) {
            case 1:
                i3 = 0 + 1;
            case 2:
                i2 = i3 + 2;
                break;
            case 3:
                i2 = 0 + 3;
                break;
            default:
                i2 = -1;
                break;
        }
        return i2;
    }

    public static void main(String[] strArr) {
        System.out.println(numSwitch(2, 10) + ":" + numSwitch(9, 10));
        System.out.println(strSwitch("beta") + ":" + strSwitch("zz"));
        System.out.println(enumSwitch(Op.MUL, 7) + ":" + enumSwitch(Op.ADD, 7));
        System.out.println(fallThrough(1) + ":" + fallThrough(3));
    }
}
