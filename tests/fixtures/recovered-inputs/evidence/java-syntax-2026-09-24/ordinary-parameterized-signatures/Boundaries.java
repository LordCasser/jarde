import java.lang.annotation.*;
import java.util.*;

public class Boundaries {
    @Retention(RetentionPolicy.RUNTIME)
    @Target(ElementType.TYPE_USE)
    public @interface Mark {}

    public static class Outer {
        public static class Inner { public int value() { return 1; } }
    }
    public static class Left { public static class Item {} }
    public static class Right { public static class Item {} }

    public static class GenericContext<T> {
        public T identity(T value) { return value; }
    }

    public static List<String> identity(List<String> values) { return values; }
    public static List<String> bodyPositive(List<String> values) { return values; }
    public static List<String> bodyOverload(List<String> values) { return values; }
    public static String choose(CharSequence value) { return "char-sequence"; }
    public static String choose(Object value) { return "object"; }
    public static String overloadedCall(List<String> values) {
        return choose(bodyOverload(values).get(0));
    }
    public static Outer.Inner inner(Outer.Inner value) { return value; }
    public static Left.Item leftItem(Left.Item value) { return value; }
    public static Right.Item rightItem(Right.Item value) { return value; }
    public static List<@Mark String> annotated(List<@Mark String> value) { return value; }

    public static void main(String[] args) throws Exception {
        System.out.println("overload=" + overloadedCall(Arrays.asList("ok")));
        System.out.println("inner=" + inner(new Outer.Inner()).value());
        System.out.println("annotated=" + annotated(Arrays.asList("ok")).get(0));
        System.out.println("class-variable=" + new GenericContext<String>().identity("ok"));
    }
}
