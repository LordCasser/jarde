package dt15;

import java.lang.reflect.Type;
import java.lang.reflect.TypeVariable;

public class Runner {
    public static void main(String[] args) {
        Bounds<Integer> bounded = new Bounds<Integer>();
        Contract<Integer> contract = null;
        TypeVariable<?> boundedParameter = bounded.getClass().getTypeParameters()[0];
        Type[] bounds = boundedParameter.getBounds();
        TypeVariable<?> contractParameter = Contract.class.getTypeParameters()[0];
        System.out.println(boundedParameter.getName() + ":" + bounds[0].getTypeName()
                + ":" + bounds[1].getTypeName());
        System.out.println(contractParameter.getName() + ":"
                + contractParameter.getBounds()[0].getTypeName());
        System.out.println(new Plain().getClass().getTypeParameters().length + ":" + (contract == null));
    }
}
