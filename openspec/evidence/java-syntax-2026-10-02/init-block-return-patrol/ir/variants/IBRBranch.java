public class IBRBranch {
    static int low;
    static int high;
    static {
        if (Boolean.getBoolean("ibr-branch-boom")) { throw new IllegalStateException("branch-fail"); }
        low = 7;
        if (Boolean.getBoolean("ibr-branch-guard")) { throw new IllegalStateException("branch-guard"); }
        high = low + 2;
    }
    public static void main(String[] args) {
        System.out.println(low + ":" + high);
    }
}
