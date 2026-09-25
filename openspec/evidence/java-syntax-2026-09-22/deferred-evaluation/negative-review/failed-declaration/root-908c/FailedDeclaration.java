public class FailedDeclaration {
    public static int run(int n) {
        int local = n << 1;
        return Support.take(local);
    }

    // Keep the helper references in this class's pool for the exact Code patch.
    public static void keepPool() {
        Support.take(1);
        Support.mark();
    }
}
