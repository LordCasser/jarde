public final class HandlerOperatorsDriver {
    public static void main(String[] args) {
        check("int normal", HandlerOperators.intOperators(-2147483643, 6, false), -36);
        check("int exceptional", HandlerOperators.intOperators(-2147483643, 6, true), -30);
        check("long normal", HandlerOperators.longOperators(-9223372036854775803L, 6L, false), -334L);
        check("long exceptional", HandlerOperators.longOperators(-9223372036854775803L, 6L, true), -328L);
        check("conversion normal", HandlerOperators.converted(4294967298L, false), 3);
        check("conversion exceptional", HandlerOperators.converted(4294967298L, true), 1);
        check("division normal", HandlerOperators.quotientBeforeTry(false, 43, 6), 8);
        check("division exceptional", HandlerOperators.quotientBeforeTry(true, 43, 6), 6);
        boolean arithmeticEscaped = false;
        try {
            HandlerOperators.quotientBeforeTry(true, 43, 0);
        } catch (ArithmeticException expected) {
            arithmeticEscaped = true;
        }
        if (!arithmeticEscaped) throw new AssertionError("division by zero was swallowed by method catch");
        System.out.println("PASS: int/long operators, conversion, handler paths, and pre-try ArithmeticException");
    }

    private static void check(String label, int actual, int expected) {
        if (actual != expected) throw new AssertionError(label + " expected=" + expected + " actual=" + actual);
    }

    private static void check(String label, long actual, long expected) {
        if (actual != expected) throw new AssertionError(label + " expected=" + expected + " actual=" + actual);
    }
}
