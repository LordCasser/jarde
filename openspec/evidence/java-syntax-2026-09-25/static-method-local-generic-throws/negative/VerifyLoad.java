package evidence;

public final class VerifyLoad {
    public static void main(String[] args) throws Exception {
        Class.forName(args[0]);
        System.out.println("loaded");
    }
}
