public class ConstructorPrimitiveHandlerControls {
    static int markInt(int value) {
        System.out.print("handler:");
        System.out.println(value);
        return value;
    }

    static Long sameHandler(int value) {
        Long result = null;
        try {
            result = new Long((long) markInt(value));
            markInt(value);
        } catch (RuntimeException failure) {
            return null;
        }
        return result;
    }
}
