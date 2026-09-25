package probe;
public class Reflect {
    public static void main(String[] args) throws Exception {
        java.lang.reflect.Method m = StaticThrows.class.getMethod("echo", Object.class);
        System.out.println("params=" + m.getTypeParameters().length);
        System.out.println("return=" + m.getGenericReturnType().getTypeName());
        System.out.println("throws=" + m.getGenericExceptionTypes()[0].getTypeName());
    }
}
