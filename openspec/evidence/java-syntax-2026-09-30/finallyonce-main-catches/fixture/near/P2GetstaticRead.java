public class P2GetstaticRead {
    private static boolean mode;
    public static void t() { if (mode) throw new IllegalStateException("state"); }
    public static void quiet() { System.out.println("quiet"); }
    public static String run() {
        System.out.println("before");
        try {
            t();
            quiet();
            return "normal";
        } catch (IllegalStateException error) {
            return "caught:" + error.getMessage();
        }
    }
    public static void main(String[] args) {
        mode = false;
        System.out.println(run());
        mode = true;
        System.out.println(run());
    }
}
