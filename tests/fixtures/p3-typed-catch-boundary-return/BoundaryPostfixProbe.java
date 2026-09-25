public final class BoundaryPostfixProbe {
    static int calls;

    static String after(String value) {
        calls++;
        throw new IllegalArgumentException("outside");
    }

    static String choosePostfix() {
        String value;
        try {
            value = "ok";
        } catch (IllegalArgumentException error) {
            return "caught";
        }
        return after(value);
    }

    public static void main(String[] args) {
        try {
            System.out.println("result=" + choosePostfix());
        } catch (RuntimeException error) {
            System.out.println("escaped=" + error.getClass().getSimpleName());
        }
        System.out.println("calls=" + calls);
    }
}
