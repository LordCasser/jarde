public class L2 {
    public static String contOuter(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (j == 2) continue outer;
                b.append(i).append(j).append(' ');
            }
        }
        return b.toString();
    }
    public static String brkOuter(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (i == 2) break outer;
                b.append(i).append(j).append(' ');
            }
        }
        return b.toString();
    }
    public static String triplePlain(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    if (k == 1) break;
                    if (j == 2) continue;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String innerLabel(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            mid:
            for (int j = 0; j < n; j++) {
                if (j == 1) continue mid;
                b.append(i).append(j).append(' ');
            }
        }
        return b.toString();
    }
    public static void main(String[] a) {
        System.out.println(contOuter(3)); System.out.println(brkOuter(3)); System.out.println(triplePlain(3)); System.out.println(innerLabel(3));
    }
}
