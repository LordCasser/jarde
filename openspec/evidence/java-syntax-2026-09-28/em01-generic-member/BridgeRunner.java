package em01;

@SuppressWarnings({"rawtypes", "unchecked"})
public class BridgeRunner {
    public static void main(String[] args) {
        Generic.A<String> value = new Generic.A<String>() {};
        long bridges = java.util.Arrays.stream(Generic.A.class.getDeclaredMethods())
                .filter(java.lang.reflect.Method::isBridge).count();
        System.out.println("type-parameters=" + Generic.A.class.getTypeParameters().length);
        System.out.println("interface=" + Generic.A.class.getGenericInterfaces()[0].getTypeName());
        System.out.println("methods=" + Generic.A.class.getDeclaredMethods().length + ":" + bridges);
        System.out.println("typed=" + value.compareTo(value));
        Comparable erased = value;
        System.out.println("bridge-null=" + erased.compareTo(null));
        try {
            erased.compareTo(new Object());
            System.out.println("bridge-wrong=accepted");
        } catch (ClassCastException expected) {
            System.out.println("bridge-wrong=ClassCastException");
        }
    }
}
