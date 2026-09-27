public class Runner {
    public static void main(String[] args) {
        System.out.println(FinallyOnce.handled(false) + ":" + FinallyOnce.count());
        System.out.println(FinallyOnce.handled(true) + ":" + FinallyOnce.count());
    }
}
