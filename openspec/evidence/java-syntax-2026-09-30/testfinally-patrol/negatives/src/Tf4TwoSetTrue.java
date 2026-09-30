public class Tf4TwoSetTrue {
    private int result = 0;
    public String test() {
        boolean success = false;
        try {
            String value = call();
            result++;
            success = true;
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
    public static void main(String[] args) {
        Tf4TwoSetTrue t = new Tf4TwoSetTrue();
        System.out.println("result=" + t.check());
    }
}
