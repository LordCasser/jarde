public final class ShiftLocalBoundaryRunner {
    public static void main(String[] args) {
        int result = ShiftLocalBoundary.savedBeforeOverwrite(7, 2);
        if (result != 28) throw new AssertionError("saved value was replaced: " + result);
        System.out.println("savedBeforeOverwrite=" + result);
    }
}
