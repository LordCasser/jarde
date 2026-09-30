public class Tf4NoSetTrue {
    private int result = 0;
    public String test() {
        boolean success = false;
        try {
            String value = call();
            result++;
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
        Tf4NoSetTrue t = new Tf4NoSetTrue();
        System.out.println("result=" + t.check());
    }
}
