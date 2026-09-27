package cf08twolvl;

public final class Runner {
    public static void main(String[] args) {
        int[][] inputs = { null, new int[0], { 3 }, { 1, 2 } };
        for (int[] input : inputs) {
            TwoLevelIf.resetCalls();
            System.out.println(TwoLevelIf.pick(input) + ":" + TwoLevelIf.calls());
        }
    }
}
