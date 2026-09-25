public final class InterfaceBoundaryRunner {
    public static void main(String[] args) {
        System.out.println("value=" + InterfaceBoundaryProbe.call()
                + ":selects=" + InterfaceBoundaryProbe.selects);
    }
}
