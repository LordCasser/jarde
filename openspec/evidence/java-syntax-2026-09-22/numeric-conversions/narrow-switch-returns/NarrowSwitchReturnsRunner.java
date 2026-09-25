public final class NarrowSwitchReturnsRunner {
    private NarrowSwitchReturnsRunner() {}

    public static void main(String[] args) {
        int[] tags = {-1, 0, 1, 2};
        for (int tag : tags) {
            System.out.println("runByte(" + tag + ")=" + NarrowSwitchReturns.runByte(tag));
            System.out.println("runChar(" + tag + ")=" + ((int) NarrowSwitchReturns.runChar(tag)));
            System.out.println("runShort(" + tag + ")=" + NarrowSwitchReturns.runShort(tag));
        }
        System.out.println("boolByte(false)=" + NarrowSwitchReturns.boolByte(false));
        System.out.println("boolByte(true)=" + NarrowSwitchReturns.boolByte(true));
        System.out.println("boolChar(false)=" + ((int) NarrowSwitchReturns.boolChar(false)));
        System.out.println("boolChar(true)=" + ((int) NarrowSwitchReturns.boolChar(true)));
        System.out.println("boolShort(false)=" + NarrowSwitchReturns.boolShort(false));
        System.out.println("boolShort(true)=" + NarrowSwitchReturns.boolShort(true));
    }
}
