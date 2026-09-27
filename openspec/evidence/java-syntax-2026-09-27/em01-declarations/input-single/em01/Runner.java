package em01;
public class Runner {
    public static void main(String[] args) {
        System.out.println(java.lang.reflect.Modifier.isAbstract(SingleAbstract.A.class.getModifiers()) + ":" + SingleAbstract.A.class.getDeclaredMethods().length);
    }
}
