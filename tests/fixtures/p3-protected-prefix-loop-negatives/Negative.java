import java.util.Iterator;
import java.util.List;

final class Negative {
    static int effects;

    static int directReturnAfterLoop(List source) {
        try {
            Iterator iterator = source.iterator();
            while (iterator.hasNext()) {
                iterator.next();
            }
            return 1;
        } catch (RuntimeException caught) {
            return -1;
        }
    }

    static void stateWriteInsideTryAfterLoop(List source) {
        try {
            Iterator iterator = source.iterator();
            while (iterator.hasNext()) {
                iterator.next();
            }
            effects += 2;
        } catch (RuntimeException caught) {
            effects++;
        }
    }
}
