public class OpAbsRunner {
    public static void main(String[] args) throws Exception {
        Class<?> enumClass = Class.forName("p.Holder2$OpAbs");
        Object[] constants = (Object[]) enumClass.getMethod("values").invoke(null);
        java.lang.reflect.Method apply = enumClass.getMethod("apply", int.class, int.class);
        for (Object constant : constants) {
            java.lang.reflect.Method name = enumClass.getMethod("name");
            java.lang.reflect.Method ordinal = enumClass.getMethod("ordinal");
            System.out.println(name.invoke(constant) + ":" + ordinal.invoke(constant) + ":" + apply.invoke(constant, 3, 4));
        }
        Object mul = enumClass.getMethod("valueOf", String.class).invoke(null, "MUL");
        System.out.println("valueOf: " + (mul == constants[1]));
    }
}
