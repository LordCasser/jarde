package cf02;

public class Predicates {
    public boolean greater(float left, float right) {
        return left > right;
    }

    public boolean mixed(float left, double right) {
        return left < right;
    }

    public boolean inBounds(int[] array, int index) {
        return index >= 0 && index < array.length;
    }

    public boolean named(Object value) {
        if (value == null || !(value instanceof String)) {
            return false;
        }
        return ((String) value).length() > 0;
    }
}
