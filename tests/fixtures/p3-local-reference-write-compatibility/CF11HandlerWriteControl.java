package cf11;

public final class CF11HandlerWriteControl {
    private static Object asObject(Object value) {
        return value;
    }

    public static Object handlerWrite() {
        Object target = "seed";
        try {
            throw new IllegalArgumentException("handler");
        } catch (IllegalArgumentException error) {
            target = asObject(error);
        }
        return target;
    }
}
