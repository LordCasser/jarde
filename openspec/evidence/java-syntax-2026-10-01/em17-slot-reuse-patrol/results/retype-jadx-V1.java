
/* JADX INFO: loaded from: V1.class */
public class V1 {
    static int side() {
        return 2;
    }

    public static String three() {
        StringBuilder sb = new StringBuilder();
        for (int i : new int[]{side(), side() + 1, side() * 2}) {
            sb.append(i).append(',');
        }
        boolean[] zArr = new boolean[3];
        zArr[side() - 1] = true;
        sb.append(zArr[1]);
        sb.append(new Object[]{"x", "y"}[1]);
        return sb.toString();
    }

    public static void main(String[] strArr) {
        System.out.println(three());
    }
}
