public class M1 {
    private static int cleanupCount;
    public static String handled(boolean fail) {
        cleanupCount = 0;
        try {
            if (fail) throw new IllegalArgumentException("arg");
            cleanupCount++;
            return "normal";
        } catch (IllegalArgumentException error) {
            cleanupCount++;
            return "caught:" + error.getMessage();
        } finally {
            cleanupCount++;
        }
    }
    public static int count() { return cleanupCount; }
    public static void main(String[] args) {
        System.out.println(handled(false) + ":" + count());
        System.out.println(handled(true) + ":" + count());
    }
}
