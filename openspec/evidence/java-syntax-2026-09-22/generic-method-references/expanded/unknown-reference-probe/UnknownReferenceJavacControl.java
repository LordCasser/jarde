import java.util.function.Function;

final class UnknownReferenceJavacControl {
    static Integer consume(UnknownLeft value) {
        return 73;
    }

    static Function<UnknownLeft, Integer> sameType() {
        return UnknownReferenceJavacControl::consume;
    }

    static Function<UnknownRight, Integer> unknownRelation() {
        return UnknownReferenceJavacControl::consume;
    }
}
