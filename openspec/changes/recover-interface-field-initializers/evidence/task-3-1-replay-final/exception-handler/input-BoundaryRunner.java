public final class BoundaryRunner {
    public static void main(String[] args) throws Exception {
        Class<?> probe = Class.forName(args[0]);
        System.out.println(probe.getMethod("observe").invoke(null));
    }
}
