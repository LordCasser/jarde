package em11;

public class Runner {
    public static void main(String[] args) {
        System.out.println(OverloadCalls.run("text"));
        System.out.println(OverloadCalls.run(new Object()));
        System.out.println(HierarchyCalls.run());
    }
}
