public class E2FieldArrayOrderDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            E2FieldArrayOrderDevB.trace = "";
            try { System.out.println(n + "|" + E2FieldArrayOrderDevB.run(n) + "|" + E2FieldArrayOrderDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E2FieldArrayOrderDevB.trace); }
        }
    }
}
