public class R2LabeledExitDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            R2LabeledExitDevB.trace = "";
            try { System.out.println(n + "|" + R2LabeledExitDevB.run(n) + "|" + R2LabeledExitDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + R2LabeledExitDevB.trace); }
        }
    }
}
