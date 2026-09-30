public class Tf4LeadExtra {
    private int result = 0;
    public String test() {
        int pad = 5;
        boolean success = false;
        try {
            String value = call();
            result++;
            success = true;
            return value + pad;
        } finally {
            if (!success) {
                result -= 2;
            }
        }
    }
    private String call() { return "call"; }
    public int check() { test(); return result; }
    public static void main(String[] args) {
        Tf4LeadExtra t = new Tf4LeadExtra();
        System.out.println("result=" + t.check());
    }
}
