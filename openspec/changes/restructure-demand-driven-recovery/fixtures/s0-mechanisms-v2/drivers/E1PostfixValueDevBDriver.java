public class E1PostfixValueDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            E1PostfixValueDevB.trace = "";
            try { System.out.println(n + "|" + E1PostfixValueDevB.run(n) + "|" + E1PostfixValueDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + E1PostfixValueDevB.trace); }
        }
    }
}
