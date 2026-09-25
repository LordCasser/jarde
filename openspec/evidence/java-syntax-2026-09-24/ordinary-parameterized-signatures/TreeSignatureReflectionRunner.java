import java.lang.reflect.Method;
import java.util.Map;
public class TreeSignatureReflectionRunner {
    public static void main(String[] args) throws Exception {
        Method m = OrdinaryParameterizedSignatures.class.getMethod("nested", Map.class);
        System.out.println(m.getGenericParameterTypes()[0].getTypeName());
    }
}
