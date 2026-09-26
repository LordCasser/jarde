public final class JardeControlProjection {
    public static String labelThrown(String label) {
        try {
            throw new java.lang.ArithmeticException(label);
        } catch (java.lang.ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static String literalThrown() {
        try {
            throw new java.lang.ArithmeticException("arm-2");
        } catch (java.lang.ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static String concatenated(String label) {
        return "arm-" + label;
    }

    public static void main(String[] args) {
        System.out.println(labelThrown("arm-2"));
        System.out.println(literalThrown());
        System.out.println(concatenated("2"));
    }
}
