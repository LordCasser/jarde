public class ClassVariableProbe<T extends Number> {
    public T choose(T left, T right, boolean first) {
        return first ? left : right;
    }
}
