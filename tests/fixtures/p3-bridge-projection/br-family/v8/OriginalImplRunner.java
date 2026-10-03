public class OriginalImplRunner {
    public static void main(String[] a) {
        BR.Impl i = new BR.Impl();
        System.out.println(i.compareTo(i));
        java.lang.Comparable c = i;
        System.out.println(c.compareTo(i));
    }
}
