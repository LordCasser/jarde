public class ConstructorPrimitiveHandlerControls {
    static int markInt(int value) {
        System.out.print("handler:");
        System.out.println(value);
        return value;
    }

    static Long identity(Long value) {
        return value;
    }

    static Long sameHandler(int value) {
        try {
            return identity(new Long((long) markInt(value)));
        } catch (RuntimeException failure) {
            return null;
        }
    }
}
