public class V1 {
    enum Op { ADD, SUB, MUL }
    public static int numSwitch(int x, int y) {
        switch (x) {
            case 1: return y + 1;
            case 2: return y * 2;
            case 5: return y - 5;
            default: return -y;
        }
    }
    public static String strSwitch(String s) {
        switch (s) {
            case "alpha": return "A";
            case "beta": return "B";
            case "gamma": return "G";
            default: return "?";
        }
    }
    public static int enumSwitch(Op o, int v) {
        switch (o) {
            case ADD: return v + 1;
            case SUB: return v - 1;
            case MUL: return v * 2;
            default: return 0;
        }
    }
    public static int fallThrough(int x) {
        int r = 0;
        switch (x) {
            case 1: r += 1;
            case 2: r += 2; break;
            case 3: r += 3; break;
            default: r = -1;
        }
        return r;
    }
    public static void main(String[] a) {
        System.out.println(numSwitch(2, 10) + ":" + numSwitch(9, 10));
        System.out.println(strSwitch("beta") + ":" + strSwitch("zz"));
        System.out.println(enumSwitch(Op.MUL, 7) + ":" + enumSwitch(Op.ADD, 7));
        System.out.println(fallThrough(1) + ":" + fallThrough(3));
    }
}
