public class TagRunner {
    public static void main(String[] args) {
        System.out.println(Tagged.class.isAnnotationPresent(Deprecated.class));
        System.out.println(Tagged.value());
        System.out.println(RetentionTagged.class.getAnnotation(java.lang.annotation.Retention.class));
    }
}
