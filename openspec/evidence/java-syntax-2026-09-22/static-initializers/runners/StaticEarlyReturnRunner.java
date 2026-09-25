public class StaticEarlyReturnRunner {
    public static void main(String[] args) {
        System.setProperty("jarde.static.early", args.length == 0 ? "false" : args[0]);
        System.out.println("value=" + StaticEarlyReturn.get());
    }
}
