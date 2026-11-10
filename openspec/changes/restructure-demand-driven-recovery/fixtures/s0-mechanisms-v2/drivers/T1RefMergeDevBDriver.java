public class T1RefMergeDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            T1RefMergeDevB.trace = "";
            try { System.out.println(n + "|" + T1RefMergeDevB.run(n) + "|" + T1RefMergeDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T1RefMergeDevB.trace); }
        }
    }
}
