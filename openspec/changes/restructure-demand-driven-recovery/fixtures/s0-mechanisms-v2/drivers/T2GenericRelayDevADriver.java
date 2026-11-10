public class T2GenericRelayDevADriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            T2GenericRelayDevA.events = "";
            try { System.out.println(n + "|" + T2GenericRelayDevA.run(n) + "|" + T2GenericRelayDevA.events); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T2GenericRelayDevA.events); }
        }
    }
}
