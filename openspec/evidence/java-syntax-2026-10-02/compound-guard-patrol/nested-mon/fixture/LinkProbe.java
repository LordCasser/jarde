public class LinkProbe {
    public static void main(String[] a) throws Throwable {
        // Link (verify) the patched class without running its broken method.
        Class.forName("NMiss", true, LinkProbe.class.getClassLoader());
        System.out.println("linked ok");
    }
}
