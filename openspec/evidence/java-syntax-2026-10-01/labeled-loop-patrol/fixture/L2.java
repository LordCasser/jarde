public class L2 {
    public static String contWithTail() {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 3; j++) {
                if (j == 1) continue outer;
                b.append(i).append(j).append(',');
            }
            b.append('T').append(i).append(';');
        }
        return b.toString();
    }
    public static void main(String[] x) { System.out.println(contWithTail()); }
}
