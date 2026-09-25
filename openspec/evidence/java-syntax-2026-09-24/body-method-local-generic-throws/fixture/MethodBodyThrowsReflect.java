package methodbodythrows;

public class MethodBodyThrowsReflect {
    public static void main(String[] args) throws Exception {
        java.lang.reflect.Method method = MethodBodyThrows.class.getMethod("run");
        System.out.println("parameters=" + method.getTypeParameters().length);
        System.out.println("throws=" + method.getGenericExceptionTypes()[0].getTypeName());
    }
}
