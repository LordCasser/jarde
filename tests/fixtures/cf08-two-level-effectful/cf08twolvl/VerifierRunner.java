package cf08twolvl;

public final class VerifierRunner {
    public static void main(String[] args) {
        int[] xs = {1};
        System.out.println(TwoLevelIfNegatives.extraEntry(xs));
        System.out.println(TwoLevelIfNegatives.differentTarget(xs));
        System.out.println(TwoLevelIfNegatives.bypassJoin(xs));
        System.out.println(TwoLevelIfNegatives.fourthJoinInput(xs));
        System.out.println(TwoLevelIfNegatives.doubleCall(xs));
        System.out.println(TwoLevelIfNegatives.withHandler(xs));
        System.out.println(TwoLevelIfNegatives.extraConsumer(xs));
    }
}
