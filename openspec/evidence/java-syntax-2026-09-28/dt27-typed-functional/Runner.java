package dt27;

public class Runner {
    public static void main(String[] args) throws Exception {
        TypedRefs refs = new TypedRefs("xy");
        System.out.println(TypedRefs.parse().apply("5") + ":" + refs.bound().apply("abc") + ":" + refs.supplier().get());
        for (String method : new String[] {"parse", "bound", "supplier"}) {
            System.out.println(method + "=" + TypedRefs.class.getDeclaredMethod(method).getGenericReturnType().getTypeName());
        }
    }
}
