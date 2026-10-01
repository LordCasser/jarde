public class OpIfaceRunner {
    public static void main(String[] args) throws Exception {
        Class<?> enumClass = Class.forName("p.OpIface");
        Object[] constants = (Object[]) enumClass.getMethod("values").invoke(null);
        java.lang.reflect.Method apply = enumClass.getMethod("apply", int.class, int.class);
        for (Object constant : constants) {
            System.out.println(enumClass.getMethod("name").invoke(constant) + ":"
                + enumClass.getMethod("ordinal").invoke(constant) + ":" + apply.invoke(constant, 3, 4));
        }
        Object add = enumClass.getMethod("valueOf", String.class).invoke(null, "ADD");
        System.out.println("valueOf: " + (add == constants[0]));
    }
}
