public class S3FloatBitsDevDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            S3FloatBitsDev.trace = "";
            try { System.out.println(n + "|" + S3FloatBitsDev.run(n) + "|" + S3FloatBitsDev.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + S3FloatBitsDev.trace); }
        }
    }
}
