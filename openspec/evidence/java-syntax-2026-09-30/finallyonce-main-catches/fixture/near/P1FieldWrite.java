public class P1FieldWrite {
    private static int marker;
    private int instance;
    private static boolean mode;
    public static void t() { if (mode) throw new IllegalStateException("state"); }
    public static void quiet() { System.out.println("quiet"); }
    public String putfieldPrefix() {
        instance = 3;
        try {
            t();
            quiet();
            return "normal";
        } catch (IllegalStateException error) {
            return "caught:" + error.getMessage();
        }
    }
    public static int putstaticPrefix() {
        marker = 7;
        try {
            t();
            quiet();
            return marker;
        } catch (IllegalStateException error) {
            return -1;
        }
    }
    public static void main(String[] args) {
        P1FieldWrite self = new P1FieldWrite();
        mode = false;
        System.out.println(self.putfieldPrefix());
        System.out.println(putstaticPrefix());
        mode = true;
        System.out.println(self.putfieldPrefix());
        System.out.println(putstaticPrefix());
    }
}
