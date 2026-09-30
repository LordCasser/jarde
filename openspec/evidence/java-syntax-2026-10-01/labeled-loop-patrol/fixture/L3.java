public class L3 {
    public static String chainPlain(int n) {
        StringBuilder b = new StringBuilder();
        b.append('a').append(n).append(';');
        for (int i = 0; i < 2; i++) {
            b.append('T').append(i).append(',');
        }
        return b.toString();
    }
    public static void main(String[] x) { System.out.println(chainPlain(7)); }
}
