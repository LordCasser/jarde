package classvars;

import java.lang.reflect.Method;

public final class NegativeRunner {
    public static void main(String[] args) throws Exception {
        String selected = args[0];
        if (selected.equals("parent")) {
            System.out.println("value=" + new ParentMismatch<String>().getClass().getName());
            System.out.println("physicalParent=" + ParentMismatch.class.getSuperclass().getName());
            System.out.println("genericParent=" + ParentMismatch.class.getGenericSuperclass().getTypeName());
        } else if (selected.equals("interface")) {
            new InterfaceMismatch<String>().run();
            System.out.println("physicalInterface=" + InterfaceMismatch.class.getInterfaces()[0].getName());
            System.out.println("genericInterface=" + InterfaceMismatch.class.getGenericInterfaces()[0].getTypeName());
        } else if (selected.equals("unbound")) {
            System.out.println("value=" + new UnboundClassVariable<String>().identity("ok"));
            Method method = UnboundClassVariable.class.getMethod("identity", Object.class);
            try {
                System.out.println("genericParameter=" + method.getGenericParameterTypes()[0].getTypeName());
            } catch (Throwable error) {
                System.out.println("genericParameterError=" + error.getClass().getName());
            }
        } else if (selected.equals("shadow")) {
            System.out.println("value=" + new MethodShadowBoundary<String>().identity("ok"));
            Method method = MethodShadowBoundary.class.getMethod("identity", Object.class);
            System.out.println("classVariables=" + MethodShadowBoundary.class.getTypeParameters().length);
            System.out.println("methodVariables=" + method.getTypeParameters().length);
            System.out.println("genericParameter=" + method.getGenericParameterTypes()[0].getTypeName());
        } else if (selected.equals("inner")) {
            OuterScopeBoundary<String> outer = new OuterScopeBoundary<>();
            OuterScopeBoundary<String>.Inner inner = outer.new Inner();
            System.out.println("value=" + inner.identity("ok"));
            Method method = OuterScopeBoundary.Inner.class.getMethod("identity", Object.class);
            System.out.println("genericParameter=" + method.getGenericParameterTypes()[0].getTypeName());
        } else {
            throw new IllegalArgumentException(selected);
        }
    }
}
