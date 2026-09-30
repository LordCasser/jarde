public class Tf4StatementAfterSetTrue {
    private int result = 0;
    public String test() {
        boolean success = false;
        try {
            String value = call();
            success = true;
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
        Tf4StatementAfterSetTrue t = new Tf4StatementAfterSetTrue();
        System.out.println("result=" + t.check());
    }
}
