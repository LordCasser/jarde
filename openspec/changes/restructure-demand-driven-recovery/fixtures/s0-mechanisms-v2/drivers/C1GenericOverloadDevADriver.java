public class C1GenericOverloadDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            C1GenericOverloadDevA.events = "";
            try { System.out.println(n + "|" + C1GenericOverloadDevA.run(n) + "|" + C1GenericOverloadDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + C1GenericOverloadDevA.events); }
        }
    }
}
