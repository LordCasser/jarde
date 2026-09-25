import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.CLASS)
@Target({ElementType.FIELD, ElementType.METHOD, ElementType.PARAMETER})
@interface BoundaryMark {
    int value();
}

public final class BoundaryTagged {
    @BoundaryMark(1)
    int field;

    @BoundaryMark(2)
    int wideAndVarargs(@BoundaryMark(3) long wideLong,
                       @BoundaryMark(4) double wideDouble,
                       @BoundaryMark(5) String... tail) {
        return (int) wideLong + (int) wideDouble + tail.length;
    }

    @BoundaryMark(6)
    int sameTypeAtDistinctPositions(@BoundaryMark(7) int first,
                                    @BoundaryMark(8) String second) {
        return first + second.length();
    }
}
