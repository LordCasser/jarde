package dt23;

public class Runner {
    public static void main(String[] args) throws Exception {
        Class<Types> type = Types.class;
        System.out.println("class=" + (type.getAnnotation(A.class).c() == type));
        System.out.println("field=" + (type.getField("field").getAnnotation(A.class).c() == type));
        System.out.println("method=" + (type.getMethod("first", int.class).getAnnotation(A.class).c() == type));
        System.out.println("param0=" + (type.getMethod("first", int.class)
                .getParameters()[0].getAnnotation(A.class).c() == type));
        System.out.println("param1=" + type.getMethod("second", int.class, int.class)
                .getParameters()[1].getAnnotation(A.class).i());
    }
}
