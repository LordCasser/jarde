public class Tf4LeadTrue {
    private int result = 0;
    public String test() {
        boolean success = true;
        try {
            String value = call();
            result++;
            success = false;
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
        Tf4LeadTrue t = new Tf4LeadTrue();
        System.out.println("result=" + t.check());
    }
}
