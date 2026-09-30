public class Tf4CleanupCall {
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
                helper();
            }
        }
    }
    private String call() { return "call"; }
    private void helper() { result += 0; }
    public int check() { test(); return result; }
    public static void main(String[] args) {
        Tf4CleanupCall t = new Tf4CleanupCall();
        System.out.println("result=" + t.check());
    }
}
