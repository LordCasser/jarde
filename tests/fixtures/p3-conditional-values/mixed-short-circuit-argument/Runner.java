public final class Runner {
    public static void main(String[] args) {
        for (int mask = 0; mask < 8; mask++) {
            MixedBooleanArgument.bValue = (mask & 2) != 0;
            MixedBooleanArgument.cValue = (mask & 4) != 0;
            MixedBooleanArgument.result = false;
            MixedBooleanArgument.bCalls = 0;
            MixedBooleanArgument.cCalls = 0;
            MixedBooleanArgument.sinkCalls = 0;
            MixedBooleanArgument.call((mask & 1) != 0);
            System.out.println(mask + ":" + MixedBooleanArgument.result + ":"
                    + MixedBooleanArgument.bCalls + ":" + MixedBooleanArgument.cCalls
                    + ":" + MixedBooleanArgument.sinkCalls);
        }
    }
}
