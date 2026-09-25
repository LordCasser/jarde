public final class InterfaceSuperRunner {
    public static void main(String[] args) {
        InterfaceSuperProbe probe = new InterfaceSuperProbe();
        System.out.println(probe.value());
        System.out.println(probe.chooseRight());
        System.out.println(probe.both());
    }
}
