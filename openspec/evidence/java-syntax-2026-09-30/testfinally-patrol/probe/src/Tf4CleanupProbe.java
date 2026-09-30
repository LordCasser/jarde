public class Tf4CleanupProbe {
    public static boolean failCall = false;
    public static boolean failCleanup = false;
    public int result = 0;
    public String test() {
        boolean success = false;
        try {
            String value = call();
            result++;
            success = true;
            return value;
        } finally {
            if (!success) {
                result -= 2;
                if (failCleanup) {
                    throw new RuntimeException("cleanup");
                }
            }
        }
    }
    private String call() {
        if (failCall) {
            throw new RuntimeException("call");
        }
        return "call";
    }
}
