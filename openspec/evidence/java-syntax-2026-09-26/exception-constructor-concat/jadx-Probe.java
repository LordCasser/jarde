
/* JADX INFO: loaded from: input.jar:Probe.class */
public final class Probe {
    public static String thrown(String label) {
        try {
            throw new ArithmeticException("arm-" + label);
        } catch (ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static String labelThrown(String label) {
        try {
            throw new ArithmeticException(label);
        } catch (ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static String literalThrown() {
        try {
            throw new ArithmeticException("arm-2");
        } catch (ArithmeticException e) {
            return e.getMessage();
        }
    }

    public static String concatenated(String label) {
        return "arm-" + label;
    }

    public static ArithmeticException constructed(String label) {
        return new ArithmeticException("arm-" + label);
    }

    public static void main(String[] args) {
        System.out.println(thrown("2"));
        System.out.println(labelThrown("arm-2"));
        System.out.println(literalThrown());
        System.out.println(concatenated("2"));
        System.out.println(constructed("2").getMessage());
    }
}
