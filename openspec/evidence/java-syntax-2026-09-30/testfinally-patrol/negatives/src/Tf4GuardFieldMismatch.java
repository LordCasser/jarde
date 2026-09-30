public class Tf4GuardFieldMismatch {
    private int result = 0;
    public static boolean failCall = false;
    public static boolean failCleanup = false;
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
    public int check() { test(); return result; }
    public static void main(String[] args) {
        Tf4GuardFieldMismatch t = new Tf4GuardFieldMismatch();
        System.out.println("result=" + t.check());
    }
}
