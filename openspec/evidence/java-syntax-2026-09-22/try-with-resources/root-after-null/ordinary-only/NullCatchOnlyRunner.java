public final class NullCatchOnlyRunner {
    public static void main(String[] args) {
        NullCatchOnly.ordinary();
        System.out.println("ordinary-catch=" + NullCatchOnly.catches());
    }
}
