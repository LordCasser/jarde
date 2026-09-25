public final class QualifierBoundaryProbe {
    public static int selects;
    public static int rightCalls;
    public static int marks;
    public static int value;
    public static boolean failSelect;

    public static void reset() {
        selects = 0;
        rightCalls = 0;
        marks = 0;
        value = 5;
        failSelect = false;
    }

    public static QualifierBoundaryProbe receiver() {
        selects++;
        if (failSelect) throw new IllegalStateException("select");
        return null;
    }

    public static int rhs() {
        rightCalls++;
        return 7;
    }

    public static int throwingRhs() {
        rightCalls++;
        throw new IllegalArgumentException("rhs");
    }

    public static void mark() {
        marks++;
    }

    public static int staticRead() {
        return receiver().value;
    }

    public static int qualifiedCallReturn() {
        return receiver().rhs();
    }

    public static int plainCallReturn() {
        receiver();
        return rhs();
    }

    public static void qualifiedCallStatement() {
        receiver().mark();
    }

    public static void plainCallStatements() {
        receiver();
        mark();
    }

    public static int qualifiedStaticWrite() {
        receiver().value = rhs();
        return value;
    }

    public static int fieldWriteWithQualifiedRhs() {
        value = receiver().rhs();
        return value;
    }

    public static int plainSequenceStaticWrite() {
        receiver();
        value = rhs();
        return value;
    }

    public static int qualifiedStaticWriteThrowingRhs() {
        receiver().value = throwingRhs();
        return value;
    }

    public static int qualifiedCallThrowingReceiver() {
        return receiver().rhs();
    }
}
