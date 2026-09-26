public final class LiteralOnlyRunner {
    public static void main(String[] args) {
        if (LiteralOnly.FIRST.value() != 7 || LiteralOnly.NEXT.value() != -2) throw new AssertionError();
        System.out.println("OK LiteralOnly");
    }
}
