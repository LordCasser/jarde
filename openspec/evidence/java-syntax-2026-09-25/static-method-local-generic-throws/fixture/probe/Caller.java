package probe;
public class Caller {
    public static void main(String[] args) {
        System.out.println(StaticThrows.<String, RuntimeException>echo("ok"));
    }
}
