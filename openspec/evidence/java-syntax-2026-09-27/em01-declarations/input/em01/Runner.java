package em01;
public class Runner {
    public static void main(String[] args) throws Exception {
        System.out.println(Shape.I.class.getDeclaredMethods().length + ":" + Shape.A.class.getDeclaredMethods().length);
        System.out.println(Generic.A.class.getTypeParameters().length + ":" + Generic.A.class.getGenericInterfaces()[0].getTypeName());
    }
}
