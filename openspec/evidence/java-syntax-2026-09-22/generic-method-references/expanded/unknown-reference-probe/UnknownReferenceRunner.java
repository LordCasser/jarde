import java.util.function.Function;

public final class UnknownReferenceRunner {
    public static void main(String[] args) {
        UnknownReferenceProbe.calls = 0;
        try {
            Function<UnknownLeft, Integer> identity =
                    (Function<UnknownLeft, Integer>) (Function) UnknownReferenceProbe.sameType();
            System.out.println(
                    "sameType:" + identity.apply(new UnknownBoth())
                            + ":calls=" + UnknownReferenceProbe.calls);
        } catch (Throwable error) {
            System.out.println(
                    "sameType:" + error.getClass().getName()
                            + ":calls=" + UnknownReferenceProbe.calls);
        }

        UnknownReferenceProbe.calls = 0;
        try {
            Function<UnknownRight, Integer> unrelated =
                    (Function<UnknownRight, Integer>) (Function) UnknownReferenceProbe.unknownRelation();
            System.out.println(
                    "unknownRelation:" + unrelated.apply(new UnknownBoth())
                            + ":calls=" + UnknownReferenceProbe.calls);
        } catch (Throwable error) {
            System.out.println(
                    "unknownRelation:" + error.getClass().getName()
                            + ":calls=" + UnknownReferenceProbe.calls);
        }
    }
}
