package classvars;

import java.math.BigInteger;
import java.lang.reflect.Method;
import java.util.Arrays;

public final class BoundedClassVariableRunner {
    public static void main(String[] args) throws Exception {
        BoundedClassVariableBoundary<BigInteger> value = new BoundedClassVariableBoundary<>();
        System.out.println("value=" + value.identity(BigInteger.valueOf(19)));
        System.out.println("classVariables=" + Arrays.toString(BoundedClassVariableBoundary.class.getTypeParameters()));
        Method method = BoundedClassVariableBoundary.class.getMethod("identity", Number.class);
        System.out.println("parameter=" + method.getGenericParameterTypes()[0].getTypeName());
        System.out.println("result=" + method.getGenericReturnType().getTypeName());
    }
}
