public class F2 {
    public static int stepTwoWithQuote(int[] data) {
        int sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            sum += data[i];
            try {
                sum += risky(data[i]);
            } catch (IllegalStateException e) {
                sum -= 1;
            }
        }
        return sum;
    }
    static int risky(int v) { if (v == 3) throw new IllegalStateException("r"); return v; }
    public static void main(String[] x) {
        int[] d = {1, 2, 3, 4, 5};
        System.out.println(stepTwoWithQuote(d));
    }
}
