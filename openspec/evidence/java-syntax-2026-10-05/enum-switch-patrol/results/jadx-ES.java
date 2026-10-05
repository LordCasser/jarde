package defpackage;

/* JADX INFO: loaded from: ES.class */
public class ES {
    static java.lang.String plain(ES.Color color) {
        switch (color.ordinal()) {
            case 0:
                return "r";
            case 1:
                return "g";
            case 2:
                return "b";
            default:
                return "?";
        }
    }

    static int withDef(ES.Color color) {
        int i;
        switch (color.ordinal()) {
            case 0:
                i = 1;
                break;
            case 1:
                i = 2;
                break;
            default:
                i = 0;
                break;
        }
        return i;
    }

    static java.lang.String nested(ES.Color color, int i) {
        switch (color.ordinal()) {
            case 0:
                switch (i) {
                    case 1:
                        return "r1";
                    default:
                        return "rN";
                }
            case 1:
                return "g";
            default:
                return "x";
        }
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + plain(ES.Color.RED) + "/" + withDef(ES.Color.BLUE) + "/" + nested(ES.Color.RED, 1) + "/" + nested(ES.Color.GREEN, 9));
    }
}
