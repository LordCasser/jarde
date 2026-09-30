public class Tf4 {
    private int result = 0;
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
    private String call() { return "call"; }
    public int check() { test(); return result; }
}
