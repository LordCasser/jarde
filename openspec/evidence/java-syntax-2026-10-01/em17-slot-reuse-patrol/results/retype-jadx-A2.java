
/* JADX INFO: loaded from: A2.class */
public class A2 {
    static int side() {
        return 2;
    }

    public static String fillCalc() {
        int[] iArr = {side(), side() + 1, side() * 2};
        StringBuilder sb = new StringBuilder();
        for (int i : iArr) {
            sb.append(i).append(',');
        }
        boolean[] zArr = new boolean[3];
        zArr[side() - 1] = true;
        return sb.toString() + zArr[1];
    }

    public static void main(String[] strArr) {
        System.out.println(fillCalc());
    }
}
