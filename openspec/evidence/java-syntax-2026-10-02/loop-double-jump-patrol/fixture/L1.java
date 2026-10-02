public class L1 {
    public static String deepLabels(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (j == 2) continue outer;
                if (i == 3) break outer;
                mid:
                for (int k = 0; k < n; k++) {
                    if (k == 1) continue mid;
                    if (k == 2) break;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String labelWhile() {
        int i = 0;
        w:
        while (i < 10) {
            i++;
            do {
                if (i % 3 == 0) continue w;
                if (i > 7) break w;
            } while (false);
        }
        return "i=" + i;
    }
    public static void main(String[] a) {
        System.out.println(deepLabels(5));
        System.out.println(labelWhile());
    }
}
