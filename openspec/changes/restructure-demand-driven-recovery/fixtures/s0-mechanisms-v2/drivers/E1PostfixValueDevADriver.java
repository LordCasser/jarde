public class E1PostfixValueDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            E1PostfixValueDevA.events = "";
            try { System.out.println(n + "|" + E1PostfixValueDevA.run(n) + "|" + E1PostfixValueDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E1PostfixValueDevA.events); }
        }
    }
}
