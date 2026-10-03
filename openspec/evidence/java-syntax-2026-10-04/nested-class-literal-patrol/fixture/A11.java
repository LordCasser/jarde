import java.lang.annotation.*;
public class A11 {
    @Retention(RetentionPolicy.RUNTIME) @interface Tag { String value() default "z"; }
    @Tag("real") static class Marked { }
    public static String refl() throws Exception {
        Tag t = Marked.class.getAnnotation(Tag.class);
        return t.value();
    }
    public static String reflCast() throws Exception {
        Annotation a = Marked.class.getAnnotation(Tag.class);
        return ((Tag) a).value();
    }
    public static boolean reflPresent() { return Marked.class.isAnnotationPresent(Tag.class); }
    public static void main(String[] x) throws Exception {
        System.out.println(refl()); System.out.println(reflCast()); System.out.println(reflPresent());
    }
}
