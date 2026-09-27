package em03;

import java.io.IOException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Parameter;
import java.util.Arrays;

public class Runner {
    public static void main(String[] args) throws Exception {
        Signatures target = new Signatures();
        System.out.println(target.named("ok", 7));
        System.out.println(target.declared(9));
        try {
            target.raises();
        } catch (IOException exception) {
            System.out.println(exception.getMessage());
        }
        Method named = Signatures.class.getMethod("named", String.class, int.class);
        Parameter[] parameters = named.getParameters();
        System.out.println(parameters[0].getName() + ":" + Modifier.isFinal(parameters[0].getModifiers())
                + ":" + parameters[1].getName() + ":" + Modifier.isFinal(parameters[1].getModifiers()));
        System.out.println(Arrays.toString(Signatures.class.getMethod("declared", int.class).getExceptionTypes()));
        System.out.println(Arrays.toString(Signatures.class.getMethod("raises").getExceptionTypes()));
    }
}
