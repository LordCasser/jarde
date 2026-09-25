public final class QualifierBoundaryRunner {
    private static void state(String label) {
        System.out.println(label + ":select=" + QualifierBoundaryProbe.selects
                + ":rhs=" + QualifierBoundaryProbe.rightCalls
                + ":marks=" + QualifierBoundaryProbe.marks
                + ":value=" + QualifierBoundaryProbe.value);
    }

    public static void main(String[] args) {
        QualifierBoundaryProbe.reset();
        System.out.println("read=" + QualifierBoundaryProbe.staticRead());
        state("read-state");

        QualifierBoundaryProbe.reset();
        System.out.println("qualified-return=" + QualifierBoundaryProbe.qualifiedCallReturn());
        state("qualified-return-state");

        QualifierBoundaryProbe.reset();
        System.out.println("plain-return=" + QualifierBoundaryProbe.plainCallReturn());
        state("plain-return-state");

        QualifierBoundaryProbe.reset();
        QualifierBoundaryProbe.qualifiedCallStatement();
        state("qualified-statement");

        QualifierBoundaryProbe.reset();
        QualifierBoundaryProbe.plainCallStatements();
        state("plain-statements");

        QualifierBoundaryProbe.reset();
        System.out.println("static-write=" + QualifierBoundaryProbe.qualifiedStaticWrite());
        state("static-write-state");

        QualifierBoundaryProbe.reset();
        System.out.println("qualified-rhs-write=" + QualifierBoundaryProbe.fieldWriteWithQualifiedRhs());
        state("qualified-rhs-write-state");

        QualifierBoundaryProbe.reset();
        System.out.println("plain-static-write=" + QualifierBoundaryProbe.plainSequenceStaticWrite());
        state("plain-static-write-state");

        QualifierBoundaryProbe.reset();
        try {
            QualifierBoundaryProbe.qualifiedStaticWriteThrowingRhs();
            state("rhs-throw-returned");
        } catch (IllegalArgumentException expected) {
            state("rhs-throw");
        }

        QualifierBoundaryProbe.reset();
        QualifierBoundaryProbe.failSelect = true;
        try {
            QualifierBoundaryProbe.qualifiedCallThrowingReceiver();
            state("select-throw-returned");
        } catch (IllegalStateException expected) {
            state("select-throw");
        }
    }
}
