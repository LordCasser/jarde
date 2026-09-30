public class T3 {
    public static String mixedTail(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (i == 0 && j == 1) continue outer;
            }
            touch(b, i);
            b.append('E').append(i).append('.');
        }
        return b.toString();
    }
    static void touch(StringBuilder b, int i) { b.append('<').append(i).append('>'); }
    public static void main(String[] x) { System.out.println(mixedTail(3)); }
}
