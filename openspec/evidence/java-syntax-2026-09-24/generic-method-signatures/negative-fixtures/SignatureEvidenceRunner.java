import java.lang.reflect.Method;
import java.lang.reflect.Type;
import java.lang.reflect.TypeVariable;

public class SignatureEvidenceRunner {
    private static String typeName(Type type) {
        return type == null ? "<null Type>" : type.getTypeName();
    }

    public static void main(String[] args) throws Exception {
        Class<?> owner = Class.forName(args[0]);
        Method method = null;
        for (Method candidate : owner.getDeclaredMethods()) {
            if (candidate.getName().equals("choose")) {
                method = candidate;
                break;
            }
        }
        if (method == null) throw new AssertionError("choose missing");
        System.out.println("loaded=verified");
        for (TypeVariable<Method> variable : method.getTypeParameters()) {
            System.out.println("variable=" + variable.getName());
            for (Type bound : variable.getBounds()) System.out.println("bound=" + bound.getTypeName());
        }
        for (Type parameter : method.getGenericParameterTypes()) {
            System.out.println("parameter=" + typeName(parameter));
        }
        System.out.println("return=" + typeName(method.getGenericReturnType()));
        for (Class<?> exception : method.getExceptionTypes()) {
            System.out.println("exception=" + exception.getName());
        }
    }
}
