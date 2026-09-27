package em01;

public class Runner {
    public static void main(String[] args) {
        System.out.println(Shape.I.class.getDeclaredMethods().length + ":"
                + Shape.A.class.getDeclaredMethods().length);
    }
}
