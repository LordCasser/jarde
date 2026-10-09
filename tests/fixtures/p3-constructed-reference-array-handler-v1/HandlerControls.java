public class HandlerControls {
    static String mark(String value) {
        System.out.print("mark:");
        System.out.println(value);
        return value;
    }

    static Object[] handled() {
        try {
            return new Object[]{new StringBuilder(mark("handler"))};
        } catch (RuntimeException ex) {
            return null;
        }
    }
}
