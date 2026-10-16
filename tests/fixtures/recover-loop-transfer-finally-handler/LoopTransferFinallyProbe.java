public class LoopTransferFinallyProbe {
    private static String run(int[] xs, boolean stop) {
        try { return Integer.toString(stop ? LoopTransferFinally.finBreak(xs) : LoopTransferFinally.finContinue(xs)); }
        catch (NullPointerException expected) { return "null"; }
    }
    private static void emit(String label, int[] xs) {
        System.out.println(label + ":" + run(xs, false) + ":" + run(xs, true));
    }
    public static void main(String[] args) {
        emit("empty", new int[]{});
        emit("one", new int[]{1});
        emit("two", new int[]{1,2});
        emit("hit", new int[]{3});
        emit("late", new int[]{1,3,4});
        emit("first", new int[]{3,1});
        emit("repeat", new int[]{3,3});
        emit("negative", new int[]{-1,2});
        emit("overflow", new int[]{Integer.MAX_VALUE});
        emit("twomax", new int[]{Integer.MAX_VALUE,Integer.MAX_VALUE});
        emit("null", null);
    }
}
