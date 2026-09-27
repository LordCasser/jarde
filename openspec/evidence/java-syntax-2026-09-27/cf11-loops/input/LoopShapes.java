public class LoopShapes {
    public static int nested(int limit) {
        int total = 0;
        for (int outer = 0; outer < limit; outer++) {
            for (int inner = 0; inner < outer; inner++) {
                total += outer + inner;
            }
        }
        return total;
    }

    public static int sequential(int limit) {
        int total = 0;
        for (int first = 0; first < limit; first++) {
            total += first;
        }
        for (int second = limit; second > 0; second--) {
            total += second;
        }
        return total;
    }

    public static void main(String[] args) {
        System.out.println(nested(5));
        System.out.println(sequential(5));
    }
}
