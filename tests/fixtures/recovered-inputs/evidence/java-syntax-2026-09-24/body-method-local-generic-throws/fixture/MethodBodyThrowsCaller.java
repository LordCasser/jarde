package methodbodythrows;

public class MethodBodyThrowsCaller {
    public static void main(String[] args) {
        MethodBodyThrows value = new MethodBodyThrows();
        value.<RuntimeException>run();
        System.out.println("called");
    }
}
