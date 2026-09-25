package negative;

import java.util.Objects;
import negative.NegativeOuter.Inner;

/* JADX INFO: loaded from: NegativeUse.class */
public final class NegativeUse {
    public static Object nestedEffects(NegativeOuter negativeOuter, int i) {
        Objects.requireNonNull(negativeOuter);
        return negativeOuter.new Inner(NegativeOuter.mark("A", NegativeOuter.mark("B", i)));
    }

    public static Object preEffect(NegativeOuter negativeOuter, int i) {
        int iMark = NegativeOuter.mark("P", i);
        Objects.requireNonNull(negativeOuter);
        return negativeOuter.new Inner(NegativeOuter.mark("A", iMark));
    }
}
