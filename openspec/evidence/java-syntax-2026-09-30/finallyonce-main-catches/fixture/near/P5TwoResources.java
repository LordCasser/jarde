public class P5TwoResources {
    private static boolean mode;
    public static java.io.ByteArrayOutputStream open() { return new java.io.ByteArrayOutputStream(); }
    public static void boom() { if (mode) throw new IllegalStateException("state"); }
    public static String two() throws Exception {
        try (java.io.ByteArrayOutputStream a = open();
             java.io.ByteArrayOutputStream b = open()) {
            a.write(1);
            b.write(2);
            boom();
            return "two:" + a.size() + b.size();
        }
    }
    public static void main(String[] args) {
        mode = false;
        try {
            System.out.println(two());
        } catch (Exception error) {
            System.out.println("caught:" + error.getMessage());
        }
        mode = true;
        try {
            System.out.println(two());
        } catch (Exception error) {
            System.out.println("caught:" + error.getMessage());
        }
    }
}
