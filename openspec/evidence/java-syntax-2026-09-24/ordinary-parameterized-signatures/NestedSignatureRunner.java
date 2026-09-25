package probe;

import java.util.Arrays;
import java.lang.reflect.Method;
import java.lang.reflect.Type;

public final class NestedSignatureRunner {
    public static void main(String[] args) throws Exception {
        if (args.length == 0 || args[0].equals("execute")) {
            System.out.println("execute=" + NestedMissingSignature.echo(Arrays.asList(new Payload())).size());
            return;
        }
        Method method = NestedMissingSignature.class.getMethod("echo", java.util.List.class);
        Type parameter = method.getGenericParameterTypes()[0];
        Type result = method.getGenericReturnType();
        System.out.println("parameter=" + parameter.getTypeName());
        System.out.println("result=" + result.getTypeName());
    }
}
