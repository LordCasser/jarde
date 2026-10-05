import java.lang.annotation.*;
public class AT {
    @Retention(RetentionPolicy.RUNTIME) @interface Mark { String v() default "d"; }   // 注解类型声明
    @Deprecated                                                                          // 内建标记
    static void old(){}
    @Mark(v = "x") static int field;                                                     // 字段注解+值
    static Class<?> klass(){ return AT.class; }                                          // Class<?> + 类字面量
    static String cname(Class<?> c){ return c.getSimpleName(); }                         // 通配符参数消费
    static int size(java.util.List<? extends Number> l){ return l.size(); }              // 上界通配符参数
    static Object first(java.util.List<? super Integer> l){ return l.get(0); }          // 下界通配符参数
    public static void main(String[] a){ old(); System.out.println(""+field+klass().getSimpleName()+cname(String.class)+size(java.util.Arrays.asList(1,2))+first(java.util.Arrays.asList(5))); }
}
