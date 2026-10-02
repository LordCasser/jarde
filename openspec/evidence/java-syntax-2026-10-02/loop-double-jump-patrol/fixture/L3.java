public class L3 {
    public static String tripleNoJump(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String tripleBrkOnly(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    if (k == 1) break;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String tripleContOnly(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                for (int k = 0; k < n; k++) {
                    if (k == 1) continue;
                    b.append(i).append(j).append(k).append(' ');
                }
            }
        }
        return b.toString();
    }
    public static String dblWhileDo() {
        int i = 0;
        while (i < 10) {
            i++;
            do { if (i == 5) break; } while (false);
        }
        return "i=" + i;
    }
    public static void main(String[] a) {
        System.out.println(tripleNoJump(2)); System.out.println(tripleBrkOnly(2)); System.out.println(tripleContOnly(2)); System.out.println(dblWhileDo());
    }
}
