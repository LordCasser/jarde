public class T2GenericRelayDevBDriver {
    public static void main(String[] args) {
        for (int n : new int[]{-2, 0, 1, 4, 9}) {
            T2GenericRelayDevB.trace = "";
            try { System.out.println(n + "|" + T2GenericRelayDevB.run(n) + "|" + T2GenericRelayDevB.trace); }
            catch (Throwable t) { System.out.println(n + "|" + t.getClass().getName() + "|" + T2GenericRelayDevB.trace); }
        }
    }
}
