public class E2FieldArrayOrderDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            E2FieldArrayOrderDevA.events = "";
            try { System.out.println(n + "|" + E2FieldArrayOrderDevA.run(n) + "|" + E2FieldArrayOrderDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E2FieldArrayOrderDevA.events); }
        }
    }
}
