import java.lang.reflect.Method;

public class GenericThrowsRunner {
    public static void main(String[] args) throws Exception {
        Method method = GenericThrowsProbe.class.getMethod("choose", Number.class);
        System.out.println(GenericThrowsProbe.choose(Integer.valueOf(3)) + "|" + method.getTypeParameters().length);
        System.out.println(method.getGenericExceptionTypes()[0].getTypeName());
    }
}
