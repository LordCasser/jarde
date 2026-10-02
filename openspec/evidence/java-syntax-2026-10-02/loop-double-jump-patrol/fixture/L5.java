public class L5 {
    public static String brkSelfContSelf(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (k == 1) break;
                    if (k == 0) continue;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static String brkSelfContMid(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (k == 1) break;
                    if (j == 2) continue;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static String dblJumpDoWhile() {
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
    public static String dblJumpDoWhilePlain() {
        int i = 0;
        while (i < 10) {
            i++;
            do {
                if (i % 3 == 0) break;
                if (i > 7) return "early";
            } while (false);
        }
        return "i=" + i;
    }
    public static void main(String[] a) {
        System.out.println(brkSelfContSelf(2)); System.out.println(brkSelfContMid(3)); System.out.println(dblJumpDoWhile()); System.out.println(dblJumpDoWhilePlain());
    }
}
