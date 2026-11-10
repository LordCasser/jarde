public class T1RefMergeDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            T1RefMergeDevA.events = "";
            try { System.out.println(n + "|" + T1RefMergeDevA.run(n) + "|" + T1RefMergeDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T1RefMergeDevA.events); }
        }
    }
}
