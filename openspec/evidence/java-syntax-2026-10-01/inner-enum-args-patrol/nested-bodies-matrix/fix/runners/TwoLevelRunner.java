public class TwoLevelRunner {
    public static void main(String[] args) throws Exception {
        Class<?> enumClass = Class.forName("p.TwoLevel$Middle$Deep");
        Object[] constants = (Object[]) enumClass.getMethod("values").invoke(null);
        java.lang.reflect.Method v = enumClass.getMethod("v");
        for (Object constant : constants) {
            System.out.println(enumClass.getMethod("name").invoke(constant) + ":"
                + enumClass.getMethod("ordinal").invoke(constant) + ":" + v.invoke(constant));
        }
    }
}
