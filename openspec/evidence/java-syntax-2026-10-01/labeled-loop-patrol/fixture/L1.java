public class L1 {
    public static String labeled() {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                if (j == 1) continue outer;
                if (i == 2) break outer;
                b.append(i).append(j).append(',');
            }
        }
        return b.toString();
    }
    public static String plainNested() {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                if (j == 1) break;
                b.append(i).append(j);
            }
        }
        return b.toString();
    }
    public static void main(String[] x) { System.out.println(labeled()); System.out.println(plainNested()); }
}
