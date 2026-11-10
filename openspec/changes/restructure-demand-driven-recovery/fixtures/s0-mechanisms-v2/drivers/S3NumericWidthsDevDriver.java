public class S3NumericWidthsDevDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            S3NumericWidthsDev.trace = "";
            try { System.out.println(n + "|" + S3NumericWidthsDev.run(n) + "|" + S3NumericWidthsDev.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + S3NumericWidthsDev.trace); }
        }
    }
}
