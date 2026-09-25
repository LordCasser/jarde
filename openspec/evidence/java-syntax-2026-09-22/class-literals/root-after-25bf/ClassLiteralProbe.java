public class ClassLiteralProbe {
    static int calls;

    static Class<?> touch(Class<?> type) {
        calls++;
        return type;
    }

    public static Class<?> reference() {
        return String.class;
    }

    public static Class<?> array() {
        return String[][].class;
    }

    public static Class<?> primitiveArray() {
        return int[][].class;
    }

    public static Class<?> self() {
        return ClassLiteralProbe.class;
    }

    public static Class<?> primitive() {
        return int.class;
    }

    public static Class<?> voidType() {
        return void.class;
    }

    public static Class<?> argument() {
        return touch(String.class);
    }
}
