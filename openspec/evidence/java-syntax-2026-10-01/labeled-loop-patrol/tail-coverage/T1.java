public class T1 {
    public static String breakTail(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (i == 0 && j == 1) continue outer;
                if (i == 2 && j == 0) break outer;
                b.append(i).append(j).append(',');
            }
            b.append('T').append(i).append(';');
        }
        return b.toString();
    }
    public static void main(String[] x) { System.out.println(breakTail(4)); }
}
