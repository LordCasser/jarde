public final class IntArgsRunner {
    public static void main(String[] args) {
        if (IntArgs.LITERAL.value() != 1 || IntArgs.FIELD.value() != 3 || IntArgs.EXPR.value() != 4) throw new AssertionError();
        System.out.println("OK IntArgs");
    }
}
