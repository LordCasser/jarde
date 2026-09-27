package dt23;

@java.lang.annotation.Retention(java.lang.annotation.RetentionPolicy.RUNTIME)
public @interface A {
    Class<?> c();
    int i() default 7;
}
