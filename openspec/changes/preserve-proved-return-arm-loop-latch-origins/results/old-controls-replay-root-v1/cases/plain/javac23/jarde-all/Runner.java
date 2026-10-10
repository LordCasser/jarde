public final class Runner {
    private static void printCase(boolean e, int n) {
        System.out.println("e=" + e + ";n=" + n
                + ";prefixWhile=" + PlainOneArmLoops.prefixWhile(e, n)
                + ";noPrefix=" + PlainOneArmLoops.noPrefix(e, n)
                + ";loopAndTail=" + PlainOneArmLoops.loopAndTail(e, n)
                + ";takenArm=" + PlainOneArmLoops.takenArm(e, n));
    }

    public static void main(String[] args) {
        printCase(true, 0);
        printCase(false, 0);
        printCase(true, 1);
        printCase(false, 1);
        printCase(true, 4);
        printCase(false, 4);
    }
}
