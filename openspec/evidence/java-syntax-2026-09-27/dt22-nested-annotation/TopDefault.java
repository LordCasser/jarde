package dt22;

public @interface TopDefault {
    float value() default 1.1f;
    int count() default 3;
}
