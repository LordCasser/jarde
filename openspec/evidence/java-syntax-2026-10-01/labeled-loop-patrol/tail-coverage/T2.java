public class T2 {
    public static String twoLabels(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++) {
            inner:
            for (int j = 0; j < n; j++) {
                int k = 0;
                while (k < n) {
                    if (j > 0 && k == 1) continue inner;
                    if (i > 0 && k == 2) continue outer;
                    b.append(j).append(k).append(',');
                    k++;
                }
                b.append('W').append(j).append(';');
            }
            b.append('O').append(i).append('.');
        }
        return b.toString();
    }
    public static void main(String[] x) { System.out.println(twoLabels(3)); }
}
