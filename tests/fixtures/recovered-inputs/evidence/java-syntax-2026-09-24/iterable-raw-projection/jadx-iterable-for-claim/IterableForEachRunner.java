import java.util.Arrays;

public final class IterableForEachRunner {
    public static void main(String[] args) {
        IterableForEach subject = new IterableForEach();
        System.out.println("multi=" + subject.test(Arrays.asList("a", "", "bc")));
        System.out.println("empty=" + subject.test(Arrays.<String>asList()));
        System.out.println("nullElement=" + subject.test(Arrays.asList("a", null, "b")));
        try {
            subject.test(null);
            System.out.println("nullContainer=none");
        } catch (RuntimeException e) {
            System.out.println("nullContainer=" + e.getClass().getSimpleName());
        }
    }
}
