public class Tf4FieldMismatch {
    private int result = 0;
    private int spare = 0;
    public String test() {
        boolean success = false;
        try {
            String value = call();
            result++;
            success = true;
            return value;
        } finally {
            if (!success) {
                spare -= 2;
            }
        }
    }
    private String call() { return "call"; }
    public int check() { test(); return result; }
    public static void main(String[] args) {
        Tf4FieldMismatch t = new Tf4FieldMismatch();
        System.out.println("result=" + t.check());
    }
}
