package em20;

public class Runner {
    public static void main(String[] args) {
        System.out.println(LocalScopes.joined(true, 4));
        System.out.println(LocalScopes.joined(false, 4));
        System.out.println(LocalScopes.loop(0));
        System.out.println(LocalScopes.loop(4));
        System.out.println(LocalScopes.synchronizedLoop(3));
        System.out.println(LocalScopes.caught(false));
        System.out.println(LocalScopes.caught(true));
    }
}
