package classvars;

import java.lang.reflect.Method;
import java.util.Arrays;

public final class InterfaceVariableRunner {
    public static void main(String[] args) throws Exception {
        ClassVariableContract<String> value = new StringContract();
        System.out.println("value=" + value.identity("interface-variable"));
        System.out.println("classVariables=" + Arrays.toString(ClassVariableContract.class.getTypeParameters()));
        Method method = ClassVariableContract.class.getMethod("identity", CharSequence.class);
        System.out.println("parameter=" + method.getGenericParameterTypes()[0].getTypeName());
        System.out.println("result=" + method.getGenericReturnType().getTypeName());
    }
}
