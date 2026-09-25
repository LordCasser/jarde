/**
 * Java 8 source for the boolean-array store evaluation-order fixture.
 * The three expressions leave decimal trace digits 1, 2, and 3.
 */
public class Order {
    static int trace;

    static int[] array(int[] value) {
        trace = trace * 10 + 1;
        return value;
    }

    static int index() {
        trace = trace * 10 + 2;
        return 0;
    }

    static int value(int value) {
        trace = trace * 10 + 3;
        if (value == 99) {
            throw new IllegalStateException("value producer");
        }
        return value;
    }

    public static int put(int[] array, int input) {
        Order.array(array)[Order.index()] = Order.value(input);
        return array[0];
    }
}
