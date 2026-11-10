public class R2LabeledExitDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            R2LabeledExitDevA.events = "";
            try { System.out.println(n + "|" + R2LabeledExitDevA.run(n) + "|" + R2LabeledExitDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R2LabeledExitDevA.events); }
        }
    }
}
