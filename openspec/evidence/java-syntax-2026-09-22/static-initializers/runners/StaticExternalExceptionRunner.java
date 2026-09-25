public class StaticExternalExceptionRunner {
    public static void main(String[] args) {
        access("first");
        access("second");
    }

    private static void access(String label) {
        try {
            System.out.println(label + "=value:" + StaticExternalException.get());
        } catch (Throwable error) {
            Throwable cause = error.getCause();
            System.out.println(label + "=" + error.getClass().getName() + ":" +
                    (cause == null ? "none" : cause.getClass().getName()));
        }
    }
}
