public final class ActualStackJoinRunner {
    private ActualStackJoinRunner() {}

    public static void main(String[] args) {
        int[] tags = {-1, 0, 1, 2};
        for (int tag : tags) {
            System.out.println("runByte(" + tag + ")=" + ((int) ActualStackJoin.runByte(tag)));
            System.out.println("runChar(" + tag + ")=" + ((int) ActualStackJoin.runChar(tag)));
            System.out.println("runShort(" + tag + ")=" + ((int) ActualStackJoin.runShort(tag)));
        }
    }
}
