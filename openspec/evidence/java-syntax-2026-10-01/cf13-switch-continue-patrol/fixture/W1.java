public class W1 {
    public static int mix(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 3) {
                case 0: total += 1; break;
                case 1: total += 2; break;
                default:
                    if (i > 4) continue;
                    total += 3;
            }
            total += 10;
        }
        return total;
    }
    public static void main(String[] x) {
        System.out.println(mix(7)); System.out.println(mix(3));
    }
}
