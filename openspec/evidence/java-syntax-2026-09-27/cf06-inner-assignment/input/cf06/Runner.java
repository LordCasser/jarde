package cf06;

public class Runner {
    public static void main(String[] args) {
        System.out.println(InnerAssignCases.lengthBranch(""));
        System.out.println(InnerAssignCases.lengthBranch("1234"));
        System.out.println(InnerAssignCases.lengthBranch("1234567"));
        InnerAssignCases probe = new InnerAssignCases();
        System.out.println(probe.run("", null));
        System.out.println(probe.run("a", ""));
        System.out.println(probe.run("b", null));
        System.out.println(probe.run("c", "d"));
    }
}
