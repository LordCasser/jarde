public class RunMiss {
    public static void main(String[] a) throws Throwable {
        try { System.out.println("missing=" + NMiss.m(3)); }
        catch (Throwable t) { System.out.println("missing threw " + t.getClass().getSimpleName()); }
    }
}
