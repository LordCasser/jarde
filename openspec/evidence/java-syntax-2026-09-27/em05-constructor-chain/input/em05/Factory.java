package em05;
public class Factory {
    public static Child choose(boolean first) {
        Child result;
        if (first) {
            result = new Child();
        } else {
            result = new Child("b");
        }
        return result;
    }
    public static Child chain(String value) { return new Child(value); }
}
