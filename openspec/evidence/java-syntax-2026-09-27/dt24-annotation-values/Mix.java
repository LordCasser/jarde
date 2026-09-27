package dt24;

@java.lang.annotation.Retention(java.lang.annotation.RetentionPolicy.RUNTIME)
public @interface Mix {
    String name();
    int num();
    float value();
    double[] doubles();
    Class<?> cls();
    Mode mode();
    Simple nested();
    int[] ints();
}
