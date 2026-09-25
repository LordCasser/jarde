package negative;

public class Replay {
    public static void main(String[] args) {
        new GenericThrows().sameClassCall();
        new GenericThrowsChild().<RuntimeException>run();
        new GenericThrowsCoreChild().<RuntimeException>run();
        System.out.println("verified");
    }
}
