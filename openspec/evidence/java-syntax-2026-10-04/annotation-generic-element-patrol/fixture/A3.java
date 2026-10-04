import java.lang.annotation.*;
public class A3 {
    @Retention(RetentionPolicy.RUNTIME)
    @interface Meta {
        Class<?> noDefault();                      // 泛型无默认
        Class<?> withDefault() default Object.class;   // 泛型 + 类字面量默认  ← A9 的形
        Class<? extends Number> boundDefault() default Integer.class;  // 通配符上界 + 默认
        Class<String>[] arrayDefault() default {};  // 泛型数组 + 空默认
    }
    @Meta(noDefault = String.class, withDefault = Long.class, boundDefault = Double.class)
    static class Target { }
    public static void main(String[] a) {
        Meta m = Target.class.getAnnotation(Meta.class);
        System.out.println(m.noDefault().getSimpleName() + "/" + m.withDefault().getSimpleName() + "/" + m.boundDefault().getSimpleName() + "/" + m.arrayDefault().length);
    }
}
