final class ArrayForeachTransfersRunner {
    public static void main(String[] args) {
        System.out.println("skip-empty=" + ArrayForeachTransfers.skipNegative(new int[0]));
        System.out.println("skip=" + ArrayForeachTransfers.skipNegative(new int[] {2, -3, 4}));
        System.out.println("stop=" + ArrayForeachTransfers.stopAtNegative(new int[] {2, -3, 4}));
        try {
            ArrayForeachTransfers.skipNegative(null);
            System.out.println("skip-null=NORMAL");
        } catch (Throwable error) {
            System.out.println("skip-null=" + error.getClass().getSimpleName());
        }
    }
}
