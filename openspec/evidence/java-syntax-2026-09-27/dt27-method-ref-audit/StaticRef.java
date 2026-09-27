package dt27;

import java.util.function.IntUnaryOperator;

class StaticRef {
    static IntUnaryOperator operator() {
        return Math::abs;
    }
}
