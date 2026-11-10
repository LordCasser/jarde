public class C1GenericOverloadDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            C1GenericOverloadDevB.trace = "";
            try { System.out.println(n + "|" + C1GenericOverloadDevB.run(n) + "|" + C1GenericOverloadDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C1GenericOverloadDevB.trace); }
        }
    }
}
