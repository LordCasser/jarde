public class Tf4Probe {
    public static boolean failCall = false;
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
