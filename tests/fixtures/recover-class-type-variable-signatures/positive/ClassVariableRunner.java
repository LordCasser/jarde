package classvars;

import java.lang.reflect.Method;
import java.util.Arrays;

public final class ClassVariableRunner {
    public static void main(String[] args) throws Exception {
        ClassVariableBoundary<String> value = new ClassVariableBoundary<>();
        System.out.println("value=" + value.identity("class-variable"));
        System.out.println("classVariables=" + Arrays.toString(ClassVariableBoundary.class.getTypeParameters()));
        Method method = ClassVariableBoundary.class.getMethod("identity", Object.class);
        System.out.println("parameter=" + method.getGenericParameterTypes()[0].getTypeName());
        System.out.println("result=" + method.getGenericReturnType().getTypeName());
    }
}
