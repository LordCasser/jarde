import java.lang.annotation.*;
public class A2 {
    @Retention(RetentionPolicy.RUNTIME)
    @interface Meta {
        Class<?> kind();                       // 通配符泛型元素
        Class<? extends Number> bound();       // 上界通配符
        Class<String> exact();                 // 具体类型实参
        String[] tags() default {};            // 数组默认值
        int n() default 1;                     // 标量默认值
    }
    @Meta(kind = String.class, bound = Integer.class, exact = String.class, tags = {"a","b"}, n = 7)
    static class Target { }
    public static void main(String[] a) throws Exception {
        Meta m = Target.class.getAnnotation(Meta.class);
        System.out.println(m.kind().getSimpleName() + "/" + m.bound().getSimpleName() + "/" + m.exact().getSimpleName());
        System.out.println(m.tags().length + "/" + m.n());
    }
}
