package em11;

/* JADX INFO: loaded from: input.jar:em11/Runner.class */
public class Runner {
    public static void main(String[] strArr) {
        System.out.println(OverloadCalls.run("text"));
        System.out.println(OverloadCalls.run(new Object()));
        System.out.println(HierarchyCalls.run());
        System.out.println(InputHierarchyCalls.run());
        System.out.println("created=" + HLeaf.created);
    }
}
