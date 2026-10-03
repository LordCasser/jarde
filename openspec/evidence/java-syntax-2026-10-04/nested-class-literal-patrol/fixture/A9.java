import java.lang.annotation.*;
import java.util.Arrays;
public class A9 {
    @Retention(RetentionPolicy.RUNTIME)
    @interface Meta { String value() default "d"; int n() default 1; Class<?> k() default Object.class; String[] tags() default {}; }
    @Deprecated
    @Meta(value = "m", n = 7, tags = {"a", "b"})
    static class Target { }
    @Meta
    static class DefTarget { }
    @Override public String toString() { return "t"; }
    @SuppressWarnings({"unchecked", "rawtypes"})
    static java.util.List rawList() { return Arrays.asList(1, 2); }
    public static void main(String[] a) throws Exception {
        Meta m = Target.class.getAnnotation(Meta.class);
        System.out.println(m.value() + ":" + m.n() + ":" + Arrays.toString(m.tags()));
        System.out.println(Target.class.isAnnotationPresent(Deprecated.class));
        System.out.println(DefTarget.class.getAnnotation(Meta.class).value());
        System.out.println(rawList().size());
    }
}
