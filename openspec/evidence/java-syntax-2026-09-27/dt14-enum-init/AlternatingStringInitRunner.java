package dt14;

public final class AlternatingStringInitRunner {
    public static void main(String[] args) {
        System.out.println("alternating=" + AlternatingStringInit.FIRST.value() + ":"
            + AlternatingStringInit.SECOND.value() + ":"
            + AlternatingStringInit.THIRD.value() + ":" + AlternatingStringInit.calls);
    }
}
