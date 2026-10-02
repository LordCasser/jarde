public class L4 {
    public static String contMidPlain(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    if (j == 2) continue;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String contMidLabel(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            mid:
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    if (j == 2) continue mid;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String brkMidPlain(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    if (j == 2) break;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String brkOuterFromDeep(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    if (i == 2) break outer;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static void main(String[] a) {
        System.out.println(contMidPlain(3)); System.out.println(contMidLabel(3)); System.out.println(brkMidPlain(3)); System.out.println(brkOuterFromDeep(3));
    }
}
