package cf06;

public final class NegativeRunner {
    public static void main(String[] args) {
        System.out.println(NegativeAssignments.interleaved("abc"));
        System.out.println(NegativeAssignments.exceptional("abc"));
        System.out.println(NegativeAssignments.exceptional(null));
        System.out.println(NegativeAssignments.loopCondition("abcdef"));
    }
}
