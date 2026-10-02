public class DJTripleJump {
    public static String tripleJump(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (k == 1) break;
                    if (k == 0) continue;
                    if (j == 2) break outer;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static void main(String[] a) {
        System.out.println(tripleJump(3));
    }
}
