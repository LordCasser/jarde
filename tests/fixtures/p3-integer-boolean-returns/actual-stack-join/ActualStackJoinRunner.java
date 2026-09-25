public final class ActualStackJoinRunner {
    public static void main(String[] args) {
        for (int tag : new int[] {-1, 0, 1, 2})
            System.out.println(tag + ":" + ActualStackJoin.runByte(tag));
    }
}
