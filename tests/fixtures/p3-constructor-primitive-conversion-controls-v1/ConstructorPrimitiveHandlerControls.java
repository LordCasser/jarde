public class ConstructorPrimitiveHandlerControls {
    static int markInt(int value) {
        System.out.print("handler:");
        System.out.println(value);
        return value;
    }

    static Long sameHandler(int value) {
        try {
            Long result = new Long((long) markInt(value));
            markInt(value);
            return result;
        } catch (RuntimeException ex) {
            return null;
        }
    }
}
