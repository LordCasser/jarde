public final class BoundaryRunner {
    public static void main(String[] args) {
        System.out.println(HiddenTarget.class.getDeclaredAnnotations().length);
        System.out.println(HiddenTarget.class.getAnnotation(HiddenTag.class));
    }
}
