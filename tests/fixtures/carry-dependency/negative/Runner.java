public final class Runner {
    public static void main(String[] args) {
        for (boolean flag : new boolean[] {false, true}) {
            CarryDependencyProbe.effects = 0;
            CarryDependencyProbe.trace = "";
            CarryDependencyRefusalProbe.receiver = null;
            CarryDependencyRefusalProbe.rejectedDependency(flag);
            System.out.println(flag + "|" + CarryDependencyProbe.effects + "|" + CarryDependencyProbe.trace + "|" + (CarryDependencyRefusalProbe.receiver != null));
        }
    }
}
