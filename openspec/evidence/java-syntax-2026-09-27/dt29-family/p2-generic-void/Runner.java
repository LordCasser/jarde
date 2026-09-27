package dt29p2;

import java.lang.reflect.Method;
import java.lang.reflect.Type;
import java.lang.reflect.TypeVariable;

public final class Runner {
    public static void main(String[] args) throws Exception {
        Method method = Setter.class.getDeclaredMethod("set", Bound.class, boolean.class);
        TypeVariable<Method>[] variables = method.getTypeParameters();
        Type[] generic = method.getGenericParameterTypes();
        boolean variableName = variables.length == 1 && variables[0].getName().equals("T");
        boolean bound = variables.length == 1
                && variables[0].getBounds().length == 1
                && variables[0].getBounds()[0] == Bound.class;
        boolean genericParameter = generic.length == 2 && generic[0] == variables[0];
        boolean erasure = method.getParameterTypes()[0] == Bound.class
                && method.getParameterTypes()[1] == boolean.class;

        Bound receiver = new Bound();
        new Setter().set(receiver, true);
        if (!variableName || !bound || !genericParameter || !erasure || receiver.calls() != 1) {
            throw new AssertionError("generic void projection changed reflection or execution");
        }
        System.out.println("p2-ok:true:true:true:true:1");
    }
}
