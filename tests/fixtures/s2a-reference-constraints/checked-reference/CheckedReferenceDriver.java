public final class CheckedReferenceDriver {
    public static void main(String[] args) {
        boolean castRejected = false;
        try {
            CheckedReference.cast();
        } catch (ClassCastException expected) {
            castRejected = true;
        }
        if (!castRejected) throw new AssertionError("cast() did not throw ClassCastException");
        if (CheckedReference.test()) throw new AssertionError("test() returned true");
        System.out.println("PASS: cast=ClassCastException; test=false");
    }
}
