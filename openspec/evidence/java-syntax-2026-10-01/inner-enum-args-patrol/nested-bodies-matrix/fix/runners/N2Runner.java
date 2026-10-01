public class N2Runner {
    public static void main(String[] args) throws Exception {
        Class<?> enumClass = Class.forName("N2$Operation");
        Object[] constants = (Object[]) enumClass.getMethod("values").invoke(null);
        java.lang.reflect.Method apply = enumClass.getMethod("apply", int.class, int.class);
        for (Object constant : constants) {
            System.out.println(enumClass.getMethod("name").invoke(constant) + ":"
                + enumClass.getMethod("ordinal").invoke(constant) + ":" + apply.invoke(constant, 3, 4));
        }
        Object plus = enumClass.getMethod("valueOf", String.class).invoke(null, "PLUS");
        System.out.println("valueOf: " + (plus == constants[0]));
    }
}
